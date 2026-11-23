# Render reverse engineering

This page records the deepest static path recovered so far. The image is heavily stripped, so the final host-present method still needs a live translated frame. The queue, worker, camera, skinning, and pose boundaries are now separated.

## Startup and device creation

| Stage | Address | RVA | Static behavior |
|---|---:|---:|---|
| Runtime startup | 0x1419F4DF0 | 0x19F4DF0 | Calls configuration and graphics initialization |
| Device initializer | 0x142478440 | 0x2478440 | Creates sgaWindowClass window, calls D3D12XboxCreateDevice, allocates XMem, creates device objects |
| Device budget setup | 0x142479690 | 0x2479690 | Selects memory and resource budget values |
| Worker registration | 0x145CABA60 | 0x5CABA60 | Registers sub_142481E90 as a worker callback |
| Frame wrapper | 0x14041E9E0 | 0x41E9E0 | Emits HAL::BeginFrame, enters frame state, and invokes several context methods |

The graphics initializer creates arrays of device-owned objects at qword_1475E7300 and qword_1475E72A0. The entries are produced through repeated indirect device calls and are later consumed by the worker path.

## Worker and submission path

sub_145CABA60 registers sub_142481E90 with the scheduler. The worker function is a large lock-free queue consumer.

Observed queue behavior in sub_142481E90:

- Reads eight lanes from ymmword_1475E6F00
- Reads a second eight-lane set from ymmword_1475E6F70
- Uses atomic counters and compare-exchange loops
- Walks variable-size records with 24-byte and 32-byte strides
- Releases or recycles completed records
- Calls sub_142482660 at two command or resource boundaries

sub_142482660 is a compact indirect submission helper.

    return object->vtable[0x118 / 8](
        object,
        1,
        &qword_1475E7300[index],
        qword_1475E0E88,
        0);

The method at byte offset 0x118 is the strongest execute or synchronization candidate. It is separate from the plane submission path below. The worker receives a queue or resource index and a device-created object from the initializer.

The next runtime trace should log this call together with the resource state of the object in qword_1475E7300[index]. If the object is a command queue, this is the correct place to bridge command completion to an OpenXR eye image. If it is a fence or internal scheduler object, follow its returned handle to the host present boundary.

## PresentX-shaped queue path

The worker contains a second indirect queue call at byte offset 0x100. The call is made with a plane count and two temporary descriptor arrays.

    queue->vtable[0x100 / 8](
        queue,
        plane_count,
        plane_array,
        present_parameters,
        0);

The record builder immediately before this call walks presentation records with an 80-byte stride, groups up to 256 records, stores source objects and plane metadata in temporary arrays, calls sub_1424827C0 to derive scaled placement values, calls sub_140A1C290 with frame-event ids 5 and 6, and clears the queue token after the call.

This is the strongest static PresentX candidate. It matches the public runtime evidence that the Xbox queue has presentation slots beyond the desktop D3D12 chain. It is still a candidate until a live queue proxy confirms the slot signature.

sub_1424827C0 converts timing and placement values into integer source and destination coordinates and writes them into per-slot presentation state. It uses qword_1475DE4B0 records with a 144-byte stride.

sub_1424826A0 rotates four presentation slots in qword_1475DF270 and qword_1468919E0. It calls the device at byte offset 0x288 with a 69888-byte presentation payload and then schedules frame events through sub_140A1C290. This is the strongest presentation-core boundary.

sub_1424828B0 updates frame-pacing history for the same four-slot ring. It adjusts the pacing origin, records the display interval, and calls sub_1424829E0.

The native stereo bridge should target this path after it can provide two eye resources. The current worker supports a plane count, but it does not prove that a single plane can carry two array views. The first implementation should expose two explicit eye resources and preserve the original plane and token contracts.

The current Leonida runtime provides a useful host-side comparison. Its Vulkan presenter accepts a back buffer or PresentX planes and blits to a Win32 swapchain. The repository's current README reports measured PresentX queue slots 30 and 32 wired to the presentation core, while the source comments still mark the exact console slot signatures as open. This makes slots 30 and 32 high-priority candidates when the translated executable is available. The host presenter is a bridge for validation. Native stereo still needs two eye render targets and per-eye camera constants before any final composition.

## Frame entry

sub_14041E9E0 has a distinct frame marker and several context methods.

| Operation | Static location |
|---|---|
| Begin marker | Context vtable +0x118, object method +0x08 with HAL::BeginFrame |
| Frame preparation | Context vtable +0xE8 then sub_14041EB50 |
| Secondary preparation | Context vtable +0xE0 then sub_14041EAF0 |
| Resource preparation | Context vtable +0xC8 then sub_14041EBD0 |
| Frame-owned object | Context vtable +0x130, object method +0x18 |
| Optional object | Context vtable +0xD0, object method +0x10 |

The frame wrapper is not the presenter. It is a reliable place to attach a trace that captures the current frame index, active command context, and eye packet before the scene begins.

## Native stereo insertion

The native stereo path should be inserted at the boundary where the camera packet becomes render constants.

1. Use HAL::BeginFrame to start a frame record.
2. Locate the camera constant upload after 0x1432309C0 and 0x14322FBD0.
3. Replace one view packet with two packets carrying OpenXR view and projection matrices.
4. Let the existing scene and worker queues record the left eye.
5. Repeat the scene for the right eye, or use a verified array-resource path.
6. Submit both eye images through xrEndFrame.
7. Keep the desktop window as a mirror surface.

Do not treat the worker queue as proof of a swap chain. The first capture must distinguish command queue execution, resource ownership, and host presentation.

## Per-ped render update

sub_1435B3B30 is a high-value per-ped render update.

- Stores the current render context at a1 + 2240
- Copies camera and object state
- Calls sub_1403B1680 for a render resource path
- Calls sub_14523BD00 to finalize pose state
- Calls sub_1427C15A0 to build skinning and draw-resource records
- Updates the final render object and calls sub_14557A810

sub_1403ADF90 is a second owner path that also calls sub_14523B680 and sub_14523BD00. The two callers cover normal and alternate ownership or render states.

## What ped_skinning_cs consumes

sub_1427C15A0 is the per-frame skinning resource builder. It walks the drawable's geometry records, resolves vertex streams, selects the skinning variant, and allocates a 144-byte record for each active draw element.

The record carries:

- skinning resource indices
- vertex stream offsets
- index or meshlet metadata
- bone or palette-related flags
- source object pointers
- per-draw buffer addresses

The function does not write the local bone transforms itself. It consumes a pose and model context prepared by the pose owner path, then builds the GPU work description for ped_skinning_cs.

## Confirmed pose buffer handoff

sub_14523B680 copies a pose object between two owners.

- Pose count is read from source offset +0x0C.
- Per-bone records are copied from source offset +0x10.
- Each record occupies 0x80 bytes.
- The complete block is copied with memcpy.
- The destination retains the count, capacity, and additional pose metadata.

sub_14523B630 maps a runtime pose identifier to a source pose buffer. It validates the identifier, reads a u16 slot, and returns:

    qword_14ABBE040 + 144 + stride * slot

sub_14523C6F0 rebuilds the GPU or render-facing bone map.

- Reads the bone count from source offset +0x0C.
- Reads one bone hash from source offset +0x04 multiplied by the index.
- Resolves the hash through sub_144DAAFC0.
- Copies four 32-byte vector blocks from the resolved record.
- Writes the blocks into an output array with a per-bone stride.

This is the first proven hash-to-pose mapping point. A VRIK override should target the source pose records before sub_14523C6F0, then allow the existing hash map and skinning builder to continue.

## Pose finalization

sub_14523BD00 finalizes the pose object before the draw-resource builder.

- Clears transient pose state at a1 + 0x880 and a1 + 0x8A0.
- Calls sub_14523C670 to rebuild four auxiliary pose arrays.
- Calls sub_14523C810 for a gated runtime update.
- Updates per-bone metadata at the pose array with a 0x80 stride.
- Reads the source pose buffer through sub_14523B630.

The write chain is therefore:

    pose source lookup
        -> sub_14523B680 pose copy
        -> sub_14523BD00 pose finalization
        -> sub_14523C6F0 hash-to-bone map
        -> sub_1427C15A0 GPU skinning record builder
        -> ped_skinning_cs

This is a stronger VRIK insertion plan than writing directly into the vertex skinning buffer. The solver should modify the local parent-relative pose records while preserving the existing hash map and GPU skinning contract.

## Still unresolved in the render path

- The exact final host present method
- The exact image resource that becomes the desktop window frame
- The camera constant-buffer upload after the pose and culling updates
- Whether the translation runtime can expose two command targets without a copy

These items require a successful translated frame or a known Xbox D3D12 ABI map.

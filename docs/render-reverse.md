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

The worker has a separate batch submission at byte offset `0x118`.

| Queue slot | Arguments observed | Static role |
|---:|---|---|
| `+0x118` | `count`, a three-lane temporary record array, `qword_1475E0E88`, flag `3` | Submits up to 256 command or resource records. The worker then invokes the per-record callbacks stored in the second and third temporary lanes. |
| `+0x100` | presentation count, `&v86`, `&v84` | Submits the presentation descriptors after placement conversion and frame-event scheduling. |

For record type `3`, the worker updates `qword_1475E0E88`, calls `0x1424826A0`, and then calls `0x142482660`. Other presentation records build the `v86` and `v84` arrays from 24-byte source blocks, call `0x1424827C0`, schedule event `5`, invoke `+0x100`, schedule event `6`, and update pacing through `0x1424828B0`. This ordering is the strongest static separation between resource submission and the host-facing presentation path.

This is the strongest static PresentX call site. It matches the public runtime evidence that the Xbox queue has presentation slots beyond the desktop D3D12 chain. The exact ABI signature still needs a live queue proxy, but the control flow and plane argument construction are now proven statically.

sub_1424827C0 converts timing and placement values into integer source and destination coordinates and writes them into per-slot presentation state. It uses qword_1475DE4B0 records with a 144-byte stride.

Its final timing helper, `0x142482BC0`, compares the target timestamp against `QueryPerformanceCounter`, sleeps for the coarse remainder, and spins through a short timing helper until the target is reached. This is a pacing boundary. An OpenXR bridge should preserve the target timestamp and completion ordering while allowing `xrWaitFrame` to own the headset cadence.

sub_1424826A0 rotates four presentation slots in qword_1475DF270 and qword_1468919E0. It calls the device at byte offset 0x288 with a 69888-byte presentation payload and then schedules frame events through sub_140A1C290. This is the strongest presentation-core boundary.

The device call at `+0x288` returns a completion count and a temporary array of 56-byte records. The core walks that array and schedules event `10` for records with type `0x10000`, or event `9` for type `0x10000`-compatible records with the zero flag. This completion record is the first static evidence that the host presenter can report work after the payload call. Capture the returned count, record type, token, and the four-slot index together with the queue submission.

sub_1424828B0 updates frame-pacing history for the same four-slot ring. It adjusts the pacing origin, records the display interval, and calls sub_1424829E0.

The native stereo bridge should target this path after it can provide two eye resources. The current worker supports a plane count, but it does not prove that a single plane can carry two array views. The first implementation should expose two explicit eye resources and preserve the original plane and token contracts.

## Presentation lifecycle

sub_142484180 is the frame-level presentation wrapper.

1. Reads the current frame token and timing state.
2. Resets or advances qword_1475E6EF0 and the four-slot qword_1475DF270 ring.
3. Calls sub_140A1C290 with frame-event id 7.
4. Selects one qword_1475DE4B0 record by the current frame slot.
5. Calls sub_1424826A0 with the selected presentation record and the active queue token.
6. Finishes the frame through sub_1424844D0.

The wrapper also clears a 128-byte frame scratch area selected by `frame_counter & 1`, updates the display interval through the runtime context, schedules event `7`, selects the active 144-byte presentation record, and calls `0x1424826A0`. `0x1424844D0` stores the active token and frame counter in a small event record before forwarding it to the runtime event helper.

sub_1424844D0 stores the active frame event state and calls the device context with an event payload. The corresponding release path polls Windows messages, schedules frame-event id 8, and releases the queue-side resources.

The renderer boundary is therefore:

    scene recording
        -> command or resource worker
        -> queue method at vtable +0x118 for resource batches
        -> queue method at vtable +0x100 with plane arrays
        -> sub_1424826A0 presentation core
        -> frame-event bookkeeping
        -> host presenter

The function address for the presentation core is 0x1424826A0. The public source uses the name PresentX for the analogous Xbox queue method. The executable itself has no PresentX import or string, so the address and the surrounding plane construction are the primary static identifiers.

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

## Camera output boundary

The camera output is assembled before the per-ped render update.

- 0x14322F5D0 clamps source camera values and writes caller fields at +0x50, +0x54, and +0xA0.
- 0x14322F8D0 combines camera state at +0x2D0, +0x2E0, +0x320, +0x330, and +0x334, then writes a four-component output vector through its R8 pointer.
- 0x143230750 updates +0x31C after reading the camera vectors.
- 0x1432307F0 updates +0x330, +0x334, and +0x320 from camera limits.

For native stereo, capture the four-component output from 0x14322F8D0 and the first render constant upload that consumes it. Apply OpenXR's per-eye tangents at that boundary. Keep the original camera state for gameplay, animation, and culling inputs.

The static call chain around this boundary is now more specific.

    0x14322FBD0
        -> select +0x314 or +0x318
        -> 0x14322F5D0 scalar clamp and caller record fields
        -> 0x14322F8D0 four-component camera output
        -> quantized record at caller +0x30

The neighboring `0x14322F800` and `0x14322FAD0` helpers normalize the delta between camera vectors and interpolate it with the camera scale at `+0x31C`. They are used for camera transitions and constrained motion. They should remain upstream of the stereo hook so the HMD pose changes only the render view.

### Camera render-object handoff

The next downstream handoff is now statically identified.

| Function | RVA | Evidence |
|---|---:|---|
| Camera owner tick | `0x142008AE0` | Updates the camera source, then calls `0x143231B30` when the render state is active |
| Render camera preparation | `0x143231B30` | Reads the camera object at owner `+0x230`, calls `0x14322F690` with a unit blend, builds command/resource records, and submits them through `0x1400D0320` and `0x1400D0610` |
| Camera packet copy | `0x1453159B0` | Copies three 16-byte blocks from the temporary camera packet into the render-owned object at `+0x30`, `+0x40`, and `+0x50` |
| Render-owned object accessor | `0x144FEBB80` | Returns the object at `a1 + 1184`, so the copied fields are at owner-relative `+1216`, `+1232`, and `+1248` |
| Camera render-object initializer | `0x1405CA410` | Initializes the render-owned camera object, clears its auxiliary vector and state fields, and sets the object state used by the preparation record |

The temporary packet passed through this path contains the four-component result of `0x14322F690`, a scalar broadcast from the source camera, and a second four-component block prepared by the render preparation function. The exact semantic names of those blocks remain runtime candidates. The copy itself is proven and gives the first practical watchpoint for a per-eye camera packet before the generic resource bookkeeping runs.

The recommended first live trace is therefore `0x142008AE0 → 0x143231B30 → 0x1453159B0`. Capture the three destination blocks, their owner pointer, and the following resource identifier. If the first block is the view or camera position, duplicate the packet for left and right eyes at this boundary. If it is a culling-only packet, continue to the first constant upload after `0x1400D0610`.

### Render-object visibility consumer

The render-owned packet has a downstream transform consumer at `0x14031B0B0`.

- It reads the render-object fields at `+0x4D0` and `+0x4F0` together with a four-vector transform basis supplied by the caller.
- SIMD multiply-add operations produce two min/max vectors in a 64-byte temporary block.
- It passes a packed handle through `0x14031BC70`, then continues through state helpers at `0x1451D7080` and `0x1451D70D0`.

This is a concrete visibility or bounds handoff. It is useful for verifying that the camera packet reached culling. It does not establish the projection matrix upload, so the stereo hook should preserve this path and continue tracing after it.

The nearby `0x1404128B0 → 0x144440C60` path is a second visibility candidate. It normalizes the three packet vectors, derives scalar limits, and constructs several vector pairs that are later consumed by resource records. Its inputs and outputs should be logged before changing any per-eye culling policy.

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

sub_14523C690 calls the mapper four times at 16-byte offsets `0`, `+0x10`, `+0x20`, and `+0x30`. The finalizer at sub_14523BD00 uses the resulting auxiliary arrays to update the per-bone metadata byte at `+0x63` in each `0x80`-byte record. This metadata step is part of the pose contract and should remain intact when the VRIK solver changes local transforms.

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

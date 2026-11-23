# Static analysis

## Method

The binary was loaded into IDA Pro 9.2 and auto-analysis was allowed to finish before decompilation. The read-only harness collected imports, technical strings, string cross-references, function boundaries, and selected Hex-Rays exports.

Addresses below are image addresses. RVAs are relative to image base `0x140000000`.

## Graphics bring-up

### Confirmed initialization chain

| Role | Address | RVA | Evidence |
|---|---:|---:|---|
| Renderer startup caller | `0x1419F4DF0` | `0x19F4DF0` | Calls configuration setup, then the graphics initializer |
| Graphics initializer | `0x142478440` | `0x2478440` | Registers `sgaWindowClass`, creates a window, calls `D3D12XboxCreateDevice` |
| Post-device setup | `0x142479690` | `0x2479690` | Selects memory and resource budgets after device initialization |
| D3D12 device import | `0x146793C18` | `0x5F93C18` | `D3D12XboxCreateDevice` |
| HAL frame entry candidate | `0x14041E9E0` | `0x41E9E0` | Calls `HAL::BeginFrame` and begins per-frame state |

`0x142478440` is the first reliable renderer boundary. It creates a Win32 window, prepares a 20-word device descriptor, calls `D3D12XboxCreateDevice`, stores the device pointer in shared globals, allocates XMem backing memory, creates multiple device-owned objects, initializes command or resource groups, and constructs the Xbox ray-tracing helper.

The call uses a real `HWND`. A desktop OpenXR host can therefore attach at the presenter boundary after the Xbox graphics ABI is provided by a translation runtime.

### Device ABI observations

The initializer uses indirect device calls at these byte offsets.

| Vtable byte offset | Static observation |
|---:|---|
| `0x40` | Initial device-owned object creation |
| `0x48` | Object or heap query |
| `0x78` | Repeated indexed device setup |
| `0xD0` | Additional indexed setup |
| `0x180` | Repeated resource creation with descriptors |
| `0x248` | Resource creation or view setup |
| `0x278` | Device-level configuration |
| `0x408` and `0x418` | Object methods on created resources |
| `0x420` | Object method used during initialization |

The exact Xbox-specific method names require a live translation layer or a known ABI map. The offsets are stable static anchors for runtime tracing.

### What is absent

The import table contains `d3d12_xs.dll`, `xg_xs.dll`, `xmem.dll`, `GameInput.dll`, `XboxBvhBuilder.dll`, `amd_fidelityfx_loader_xbox.dll`, and other platform modules. No OpenXR, OpenVR, SteamVR, `xrCreateInstance`, `xrBeginFrame`, or desktop DXGI swap-chain import was found.

The literal `PresentX` was not found in the executable. The presenter must therefore be found by tracing the object created by the graphics initializer and the first successful frame submission. A source-level name cannot be inferred from the stripped image.

## Camera data path

### Confirmed camera candidates

| Role | Address | RVA | Evidence |
|---|---:|---:|---|
| Camera settings group registration | `0x145A44800` | `0x5A44800` | Registers `CameraSetting` with other settings groups |
| Setting group hash resolver | `0x143248C60` | `0x3248C60` | Maps the `CameraSetting` group to an object offset |
| Camera state event handler | `0x142650D40` | `0x2650D40` | Handles `Camera_Move`, state transitions, and direction math |
| Camera view update | `0x1432309C0` | `0x32309C0` | Reads source transform data and writes yaw and camera state fields |
| Camera source attach | `0x143230AE0` | `0x3230AE0` | Attaches a camera source at object offset `+0x2F0` |
| Camera state writer | `0x14322FBD0` | `0x322FBD0` | Writes a normalized camera output record through helper calls |
| Camera reset and default state | `0x143230C00` | `0x3230C00` | Clears camera state and applies default values |

`sub_1432309C0` writes values into the camera object at offsets `+0x2E0`, `+0x318`, and `+0x328`, and sets a dirty flag at `+0x348`. The function derives angles from source vectors and applies a vertical constraint through a source object at `+0x2C0`.

`sub_14322FBD0` selects a field at offset `+0x314` or `+0x318` depending on a camera flag, passes the value through the camera output helpers, and stores a packed result in a caller-owned record. These fields are high-value runtime watchpoints for FOV and projection work. Their semantic names are still candidates until a frame capture or live trace confirms the matrix upload.

### Projection strings

The following data labels have code or table references.

- `projectionCenter`
- `perspectiveProjection`
- `PerspectiveProjection`
- `fieldOfView`
- `resolve_mirror_reprojection`

The projection labels are referenced by configuration tables. They identify the engine vocabulary but do not prove the final matrix writer. The next static step is to follow the camera output record into the render graph and identify the constant-buffer upload.

## Visibility and skinning

The image initializes named render subsystems.

| Subsystem | Address | RVA | Evidence |
|---|---:|---:|---|
| Instance culling registration | `0x145700770` | `0x5700770` | Registers `InstanceCulling` |
| Meshlet culling setup | `0x145B03CB0` | `0x5B03CB0` | Registers `meshlet_culling` and creates related GPU resources |
| Portal stencil path | `0x140CCC430` | `0xCCC430` | Registers `portal_prepass_stencil_culling` |
| Mirror resolve path | `0x14374F390` | `0x374F390` | References `resolve_mirror_reprojection` and portal culling |
| Ped skinning setup | `0x1427C0400` | `0x27C0400` | Registers `ped_skinning_cs` and its resource descriptors |
| BVH skinning setup | `0x140CC5160` | `0xCC5160` | Registers `bvh_skinning` and `bvh_skinning_smh` |

`ped_skinning_cs` is the strongest static anchor for the final bone-to-vertex consumer. The function creates descriptors for multiple shader resources and initializes tables for the skinning compute path. It does not expose the CPU pose writer by name.

`InstanceCulling`, `meshlet_culling`, and `portal_prepass_stencil_culling` must all receive stereo-aware visibility inputs. A stereo patch that changes only the camera matrix will produce eye-dependent holes when the original single-eye cull rejects an object.

## Static conclusions

1. The renderer is created inside the executable through a native Xbox D3D12 device call.
2. The image has no built-in OpenXR bridge.
3. A native stereo port needs a host presenter, per-eye view and projection constants, stereo-aware visibility, and a compositor submission path.
4. The camera object and the skinning compute setup are statically located.
5. The CPU pose write and the final frame-present call still need runtime tracing.

## New render and pose evidence

The graphics worker path is now separated from initialization. 0x145CABA60 registers 0x142481E90 as a worker callback. The worker drains eight-lane command queues and calls 0x142482660 at command or resource boundaries. 0x142482660 invokes a context vtable method at byte offset 0x118 with an entry from qword_1475E7300. This is the strongest submission or synchronization candidate and needs a live frame to distinguish queue execution from host presentation.

The per-ped render update at 0x1435B3B30 calls pose finalization at 0x14523BD00 and then builds skinning draw records through 0x1427C15A0. A second owner path at 0x1403ADF90 reaches the same pose finalizer.

The pose copy at 0x14523B680 reads a count at source offset +0x0C and copies 0x80-byte records from +0x10. The bone mapper at 0x14523C6F0 resolves each bone hash through 0x144DAAFC0 and copies four 32-byte vector blocks. The VRIK solver should modify this local pose block before the mapper runs.

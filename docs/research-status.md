# Research status

## Confirmed

- IDA Pro 9.2 database exists and has completed analysis.
- The PE is x64 with image base `0x140000000`.
- Renderer initialization creates a window and calls `D3D12XboxCreateDevice`.
- Camera setting, camera state, culling, mirror, skinning, motion matching, and pose matching anchors are present.
- The camera helper chain now shows interpolation between `+0x2D0` and `+0x2E0`, constrained height at `+0x320`, and a four-component output at `0x14322F8D0`.
- Pose finalization now shows four auxiliary hash-mapped arrays and a per-bone metadata byte at `+0x63` within the `0x80`-byte record stride.
- The camera render-object packet now has a downstream AABB consumer at `0x14031B0B0` and a frustum/visibility-shape path at `0x1404128B0 → 0x144440C60`.
- The read-only IDA harness has been verified against the current database and exports 53 static VR, physics, input, and weapon targets with xrefs.
- The Rust runtime probe frame validator covers the observed queue and presentation invariants.
- The Rust OpenXR frame coordinator, version gate, and typed live hook order are implemented and unit-tested.
- Physics-related static anchors now cover vehicle deformation, explosion damage buffers, GPU particle/rain collision, water simulation passes, constraint naming, and horse collision response.
- The rigid-body path now includes shape-batch preparation, weighted center accumulation, second-moment or inertia accumulation, inverse-mass recomputation, and the dynamic state handoff.
- The executable has no OpenXR, OpenVR, or SteamVR import or string.
- A curated technical string table is available in [strings.md](strings.md).

## Candidate

- `0x1432309C0` and `0x14322FBD0` are camera state and normalized output tracing candidates. The final projection matrix upload remains downstream.
- The presenter is downstream of the graphics object created by `0x142478440`.
- `ped_skinning_cs` is the final bone-to-vertex consumer.
- `GoalRotationOffsetsKeyedArray` and the pose matcher resources are useful VRIK integration anchors.

## Unresolved

- Exact PresentX ABI signature and host swapchain or image handoff
- Exact view and projection constant-buffer upload
- The final semantic mapping of the frustum vector fields and the GPU resource binding that consumes them
- Source pose producer and the semantic names of the four mapped vector blocks
- Per-eye treatment of temporal, mirror, portal, and screen-space passes
- Proof that the translated runtime can boot this executable
- Physics world owner, collision query ABI, solver tick, and gameplay contact result structures

## Next experiments

1. Capture one translated frame and identify the final present boundary.
2. Watch the camera output record while changing FOV and headset pose.
3. Track the resource bound to `ped_skinning_cs` and diff it after one hand movement.
4. Implement a two-eye OpenXR smoke path with solid-color eye images.
5. Replace solid colors with the native scene after the presenter and culling contracts are measured.

## Public research boundary

The repository publishes derived addresses, aggregate facts, technical labels, and authored VR integration designs. It does not publish the executable, package resources, keys, full decompiled functions, or extracted game data.

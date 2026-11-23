# Research status

## Confirmed

- IDA Pro 9.2 database exists and has completed analysis.
- The PE is x64 with image base `0x140000000`.
- Renderer initialization creates a window and calls `D3D12XboxCreateDevice`.
- Camera setting, camera state, culling, mirror, skinning, motion matching, and pose matching anchors are present.
- The camera helper chain now shows interpolation between `+0x2D0` and `+0x2E0`, constrained height at `+0x320`, and a four-component output at `0x14322F8D0`.
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
- Source pose producer and the semantic layout of the four mapped vector blocks
- Per-eye treatment of temporal, mirror, portal, and screen-space passes
- Proof that the translated runtime can boot this executable

## Next experiments

1. Capture one translated frame and identify the final present boundary.
2. Watch the camera output record while changing FOV and headset pose.
3. Track the resource bound to `ped_skinning_cs` and diff it after one hand movement.
4. Implement a two-eye OpenXR smoke path with solid-color eye images.
5. Replace solid colors with the native scene after the presenter and culling contracts are measured.

## Public research boundary

The repository publishes derived addresses, aggregate facts, technical labels, and authored VR integration designs. It does not publish the executable, package resources, keys, full decompiled functions, or extracted game data.

# VR port completion matrix

This matrix is the current audit of the requested preparation work. A runtime item remains open until a translated process produces direct evidence.

| Requirement | Evidence | Status |
|---|---|---|
| IDA database and complete analysis | IDA 9.2 database, image base `0x140000000`, 434,462 functions, Hex-Rays enabled | Verified |
| Native graphics initialization | `0x2478440` and `D3D12XboxCreateDevice` import | Verified statically |
| Frame and worker path | `0x41E9E0`, `0x2481E90`, queue slots `+0x118` and `+0x100` | Verified statically |
| Presentation lifecycle | `0x2484180`, `0x24826A0`, payload `69888`, completion records and pacing | Verified statically, ABI runtime gate open |
| Camera output | `0x322FBD0`, `0x322F8D0`, camera render packet `0x53159B0` | Verified statically |
| Asymmetric projection | OpenXR tangent math and per-eye packet types | Pure Rust tested, engine matrix convention runtime gate open |
| Frustum and culling | `0x4128B0`, `0x4440C60`, CPU range chain, `0x25D8590` | Verified statically, GPU consumer runtime gate open |
| Pose and VRIK handoff | `0x523B630`, `0x523B680`, `0x523BD00`, `0x523C690`, `0x523C6F0` | Verified statically, source producer and vector semantics runtime gate open |
| Decoupled aim | PrecisionAimContext, Muzzle, Wpn_LOC_Projectile anchors | Verified statically, launch/hit resolver runtime gate open |
| Physics resources | Vehicle deform, explosion damage, GPU particles, rain, water, constraints, horse response | Verified statically, solver/contact ABI runtime gate open |
| OpenXR frame coordination | `gta6-vr-openxr` coordinator and two-eye tests | Unit-tested |
| Version and hook gating | `manifest_matches`, `LIVE_HOOK_ORDER`, image base and SHA-256 | Unit-tested |
| Static harness and strings | 53 targets with xrefs, 80 selected strings, 40 render candidates | Verified on the current IDA database |
| Runtime trace schema | `RuntimeProbeFrame`, bounded JSON schema, x64dbg attach map | Unit-tested and documented |
| External boot trigger | Public repositories currently report no playable boot | Pending external state |

## Release gate

The native stereo scene path becomes ready for implementation after one translated frame proves the presentation ABI, one camera constant consumer, one eye-resource handoff, and one completion signal. VRIK and decoupled aim remain behind those stereo and ownership checks.

# x64dbg live attach map

This is a read-only attach order for the first translated process. It assumes the module image base is checked before applying the RVAs. It does not launch a binary or modify process memory.

## Attach order

| Order | RVA | Breakpoint role | Read-only data |
|---:|---:|---|---|
| 1 | `0x2478440` | Device initialization | device pointer, HWND, created object pointers |
| 2 | `0x2008AE0` | Camera owner tick | camera owner, active source, render-state flag |
| 3 | `0x3231B30` | Camera render preparation | owner `+0x230`, temporary packet, resource record |
| 4 | `0x53159B0` | Render packet copy | destination object and three 16-byte blocks at `+0x30/+0x40/+0x50` |
| 5 | `0x4128B0` | Frustum preparation | packet `+0x4D0/+0x4E0/+0x4F0`, scalar fields `+0x500/+0x504/+0x508` |
| 6 | `0x4440C60` | Visibility-shape math | normalized vectors, derived fields `+0x70…+0xE0`, range scalars |
| 7 | `0x4E296B0` | CPU range dispatch | input vectors, `a2+0x20D0`, range list at `+192/+196/+200` |
| 8 | `0x25D8590` | Instance-culling batch | pass id, compact index count, culling context `+0xF0` |
| 9 | `0x2481E90` | Worker queue | record type, batch count, slot `+0x118`, plane slot `+0x100` |
| 10 | `0x24826A0` | Presentation core | four-slot index, token, payload object |
| 11 | `0x2484180` | Presentation lifecycle | frame token, timing, slot record |
| 12 | `0x2482BC0` | Presentation pacing | target timestamp and QPC wait |
| 13 | `0x523B680` | Pose copy | source/destination, count `+0x0C`, `0x80` stride |
| 14 | `0x523BD00` | Pose finalization | auxiliary destinations, metadata byte `+0x63`, gated state |
| 15 | `0x523C6F0` | Bone hash mapping | hash, resolved record, four auxiliary block indices |
| 16 | `0xCBF1F0` | Vehicle deform resource | resource handle, buffer handles, 32-slot staging table |
| 17 | `0x2511AB0` | Explosion damage buffers | vehicle/ped buffer handles and allocation sizes |
| 18 | `0x1AF3BA0` | GPU particle collision registration | collision job handle and dispatch table |
| 19 | `0x1D87EF0` | Rain collision resource | weather owner, rain descriptor, persistent resource |
| 20 | `0x1E490F0` | Water simulation target | compressed simulation target and water settings |
| 21 | `0x26205C0` | Water system state | mode code, height/depth resources, pass handles |
| 22 | `0x26870F0` | Constraint name resolver | `constraint_XX` name and resolved index |
| 23 | `0x1D8C9B0` | Horse collision response | response record and resource vtable |
| 24 | `0x645740` | Body-shape batch update | batch count `a1+0x0C`, entry range `+0x20/+0x22`, shape records with 80-byte stride |
| 25 | `0x4F5B770` | Body aggregate preparation | weighted vector inputs, aggregate output, second-moment helper calls |
| 26 | `0x4F5B8C0` | Body second-moment accumulation | center-relative vectors, per-element weights, accumulated 16-byte state |
| 27 | `0x1F97990` | Rigid-body dynamic update | body type byte `+0x155`, state flag `a1+0x54`, 56-byte temporary |
| 28 | `0x517E960` | Body state normalization | input vector, body `+0x40/+0x50`, state index `+0x60` |
| 29 | `0x53808B0` | Inverse-mass recompute | mass `+0x4C`, inverse mass `+0x5C` |
| 30 | `0x2687A20` | Shape-property update | body shape count `+0x164`, shape pointer `+0x170`, transformed 64-byte blocks |
| 31 | `0x55B1160` | Weighted center accumulation | per-shape weights, total weight, normalized center vector |
| 32 | `0x55B1200` | Inertia or second-moment accumulation | center-relative shape vectors and weighted quadratic terms |
| 33 | `0x6D9FF0` | Body property initializer | shape count, center/inertia aggregates, mass and buoyancy accumulation |
| 34 | `0x5F3FED0` | GameInput interface creation | returned input object and vtable pointer |
| 35 | `0x59258A0` | GameInput wrapper | interface replacement, initialization calls, stored global pointer |
| 36 | `0x5F34810` | Weapon event dispatcher | `action_id`, `phase`, weapon state flags, resource pointer |

## First capture sequence

1. Verify the translated module image base and record the local build hash.
2. Capture one mono frame through order `1` to `12`.
3. Capture one pose update through order `13` to `15` without changing pose bytes.
4. Compare the camera packet, frustum range list, culling count, plane resource, completion token, and pose metadata by frame id.
5. Add OpenXR eye packets only after the mono sequence is stable.
6. For physics, capture registration first and then follow one event into its update or damage consumer. Registration alone does not prove a solver call. For a rigid body, correlate orders `24` through `29` with the same body pointer and frame or tick marker.

## Read-only guardrails

- Do not patch instructions, registers, resource states, or pose bytes during the first capture.
- Keep one process instance and one debugger session.
- Store raw traces privately using [runtime-probe-schema.md](runtime-probe-schema.md).
- Stop on a reused token, missing completion record, invalid pose stride, or a range-list count that exceeds the observed batch limits.

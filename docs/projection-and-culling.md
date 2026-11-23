# Projection and culling

## Asymmetric projection

The executable has `projectionCenter`, `perspectiveProjection`, `PerspectiveProjection`, and `fieldOfView` labels. The labels are useful search anchors. They are not enough to assign a field offset without a runtime watchpoint.

OpenXR supplies four tangents for each eye.

    left  = tan(angleLeft)
    right = tan(angleRight)
    up    = tan(angleUp)
    down  = tan(angleDown)

For a D3D-style depth range, construct the off-axis projection from those tangents and the chosen near and far planes.

    width  = right - left
    height = up - down

    m00 = 2 / width
    m11 = 2 / height
    m20 = (right + left) / width
    m21 = (up + down) / height

The signs and depth terms must follow the engine's matrix convention. Confirm row versus column storage and the depth range from a captured constant buffer before writing the final matrix helper.

The projection center is the natural place to represent lens asymmetry. Do not change only a global FOV scalar. The per-eye tangents determine both FOV and optical center.

## Camera output path

The static camera path has a narrower output boundary than the settings labels.

| Function | RVA | Static behavior |
|---|---:|---|
| Camera output builder | 0x14322F5D0, RVA 0x322F5D0 | Clamps source values and writes camera output fields at caller offsets +0x50, +0x54, and +0xA0 |
| Camera output finalizer | 0x14322F8D0, RVA 0x322F8D0 | Combines camera state at +0x2D0, +0x2E0, +0x320, +0x330, +0x334 and writes a four-component output vector through the R8 pointer |
| View state update | 0x143230750, RVA 0x3230750 | Reads +0x2D0 and +0x2E0, calls scalar camera helpers, and writes +0x31C |
| Clip and limit update | 0x1432307F0, RVA 0x32307F0 | Updates +0x330, +0x334, and +0x320 from camera limits |

The output finalizer is the first strong asymmetric-projection watchpoint. A Rust hook can capture the original four components, preserve gameplay and cinematic values, and replace only the eye-dependent projection terms in the downstream render packet. The function itself does not prove the final matrix convention, so the next validation still needs one constant-buffer capture.

## Recovered camera math

The surrounding helpers clarify which values belong to camera motion and which values are candidates for the render packet.

| Function | Static operation | VR consequence |
|---|---|---|
| `0x14322F690` | Interpolates between the two 4-component camera vectors at `+0x2D0` and `+0x2E0` using a caller scalar and the height value at `+0x320`. A constrained branch clamps against `+0x334`, `+0x2E8`, and the source-camera limits at `+0xC4` and `+0xBC`. | Preserve this canonical camera position and orientation input. Apply eye offsets after the interpolation so head IPD does not feed back into gameplay state. |
| `0x14322F8D0` | Builds the final 4-component output. With flag `0x4000` it linearly interpolates the two camera vectors. Otherwise it selects the vector at `+0x2D0` or `+0x2E0` according to flag `0x4`, then derives the fourth component from `+0x320` and the constrained scalar. | Capture this output and the first constant upload that consumes it. This is the best place to split one camera packet into two eye packets. |
| `0x14322FBD0` | Selects the scalar at `+0x314` or `+0x318` based on flag `0x10`, calls the output builder and finalizer, then quantizes the resulting vector using a 32-unit scale before storing it at caller `+0x30`. | Treat the quantized record as a downstream camera-state or culling input. Do not replace it with the headset projection matrix. |
| `0x14322F800` / `0x14322FAD0` | Normalize the delta between the finalizer output and a second camera vector, scale it by `+0x31C`, and interpolate with the caller scalar. `0x14322FAD0` has a special mode that divides the scale by the source-camera value at `+0xC8` plus one. | These are transition and camera-motion helpers. Hooking here risks changing gameplay or cinematic interpolation. Use them as trace points, not as the stereo insertion boundary. |

The immediate constants used by this path include `1.0f` for the interpolation blend, `-0.5f` in the normalization fallback, and a small positive epsilon for zero-length checks. Their presence supports the interpretation above, but it does not establish the engine's matrix row or depth convention. That convention still comes from a captured render constant.

## FOV handling

`fieldOfView` is a data label referenced by configuration tables. The camera state functions also write angle-like values at `+0x318` and `+0x328`.

The implementation should keep three values separate.

1. Gameplay or cinematic FOV
2. Per-eye optical FOV from OpenXR
3. Render target crop and viewport

Gameplay FOV controls design intent. OpenXR tangents control the projection matrix. The viewport controls pixels. Mixing them creates zoom, incorrect convergence, or a stretched HUD.

The first runtime trace should log the camera output record, the projection constant buffer, and the viewport for a stationary scene while the headset rotates.

The render-owned handoff provides an additional static trace target. The active camera owner at `0x142008AE0` calls `0x143231B30`, which prepares a resource record and reaches `0x1453159B0`. That copier writes three vector-sized blocks at render-object offsets `+0x30`, `+0x40`, and `+0x50`. Watch these blocks together with the later constant-buffer bind. This separates camera-state mutation from the packet that the render graph actually receives.

The first downstream bounds consumer is `0x14031B0B0`. It reads render-object `+0x4D0` and `+0x4F0`, combines them with a caller transform basis, and produces min/max vectors for a visibility object. Trace its packed handle and the `0x14031BC70` result while changing only the HMD orientation. A changed AABB or handle confirms that camera packet data has reached culling.

### Frustum and visibility-shape candidate

Two adjacent functions provide a stronger static culling candidate.

| Function | RVA | Static evidence |
|---|---:|---|
| Frustum preparation | `0x1404128B0` | Resets the render object, reads `+0x4D0`, `+0x4E0`, `+0x4F0`, and scalar parameters at `+0x500`, `+0x504`, and `+0x508`, then builds multiple vector pairs for downstream resource records |
| Shape math | `0x144440C60` | Reads packet vectors at `+0x30`, `+0x40`, `+0x50` and scalar limits at `+0xF4` and `+0xF8`, normalizes them, writes derived scalars at `+0x100` and `+0x104`, and stores derived vector fields from `+0x70` through `+0xE0` |

The SIMD operations include cross-product-like shuffles, normalization, min/max plane construction, and reciprocal-length correction. This is the best static frustum or visibility-shape candidate found so far. It is still distinct from a proven GPU projection constant upload. The first runtime trace should capture its input packet and the generated plane vectors before the instance-culling dispatch.

After the vector generation, trace the resource wrapper at `0x14559A400` and the flag-based dispatch at `0x144E29640`. Record the resource flags, selected method, and completion callback. This is the shortest static-to-runtime path currently available for locating the GPU buffer or descriptor that consumes the frustum packet.

## Culling

The image contains explicit `InstanceCulling`, `meshlet_culling`, and portal stencil culling paths.

### Instance-culling batch path

The registration function at `0x145700770` is followed by a concrete batch builder at `0x1425D8590`.

| Function | RVA | Static behavior |
|---|---:|---|
| Instance-culling registration | `0x145700770` | Registers the `InstanceCulling` resource and its configuration fields |
| Instance-culling batch builder | `0x1425D8590` | Walks 528-byte group records, iterates up to 16 lanes per group, filters candidate ranges, writes a compact index list, and submits the list through a context vtable method at byte offset `+0xF0` |
| Range accumulator | `0x1455BFC50` | Adds a candidate offset to `a4[2]`, clamps the accumulated low and high bounds in `a4[3]` and `a4[4]`, and checks against `a4[16]` |

The `+0xF0` method belongs to the culling or command context. It is distinct from the presentation queue method at `+0x100`. The batch builder also reaches context methods at `+0x60`, `+0x88`, and `+0x160` while publishing the generated list. The static path does not expose an eye index, so the first stereo patch should feed a conservative union frustum into the batch inputs and keep the generated list shared. Per-eye lists can be introduced after a live trace shows an eye-aware resource or pass identifier.

### Broad phase

Build a conservative union frustum from both eye frusta. Use it for instance and portal broad-phase tests. This avoids dropping an object that is visible in only one eye.

For the instance path, trace the input context passed to `0x1425D8590` and the range state consumed by `0x1455BFC50`. The first useful runtime evidence is the list count, the low and high range values, the pass identifier, and the resource bound to the `+0xF0` submission. A change in list count when only the HMD rotates indicates that the camera packet has reached culling.

### Fine phase

Use per-eye frustum tests for meshlets and screen-space features when the engine supports an eye index. If the existing meshlet path cannot accept two frusta, use the union frustum and accept extra work until a direct per-eye path is traced.

### Shadows and reflections

Shadows can use a shared light-space cull. Reflections, mirrors, portals, and decals need their own eye policy. `resolve_mirror_reprojection` and `portal_prepass_stencil_culling` are explicit static anchors for this audit.

### Temporal data

Every temporal buffer needs an eye key. History from one eye cannot feed the other eye without an intentional shared-history design.

## Verification

- Compare the left and right view origins against the calibrated IPD.
- Place a thin object at several depths and check that the disparity changes sign correctly.
- Rotate only the headset and confirm that camera orientation changes without changing gameplay camera state.
- Turn on debug bounds and verify that both eyes retain the same visible world set.
- Compare union-frustum overdraw with per-eye meshlet culling after correctness is established.

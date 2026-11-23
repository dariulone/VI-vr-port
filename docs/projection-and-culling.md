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

## Culling

The image contains explicit `InstanceCulling`, `meshlet_culling`, and portal stencil culling paths.

### Broad phase

Build a conservative union frustum from both eye frusta. Use it for instance and portal broad-phase tests. This avoids dropping an object that is visible in only one eye.

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

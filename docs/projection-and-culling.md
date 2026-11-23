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

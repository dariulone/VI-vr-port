# Native stereo plan

This plan uses native per-eye rendering through OpenXR. It keeps the engine render graph and translation runtime in the loop. Reprojection-based presentation is outside the design.

## Integration boundary

The first confirmed graphics boundary is `0x142478440` at RVA `0x2478440`. It creates a window and a D3D12 device. The runtime work should attach in this order.

1. Identify the first created command queue or presenter object from the device initializer.
2. Trace the first frame entry from `HAL::BeginFrame` at `0x14041E9E0`.
3. Trace the final command submission and the object method that hands a completed color image to the host.
4. Confirm whether the translation runtime exposes an array texture, two textures, or a desktop present image.
5. Add an OpenXR session beside that presenter. Keep the game window as a mirror only.

The important rule is to preserve the engine's command recording. The OpenXR layer should provide view constants and target images at the point where the engine binds them.

## OpenXR frame loop

The host-side flow is:

    xrWaitFrame
    xrBeginFrame
    xrLocateViews
    acquire left and right swapchain images
    wait for the acquired images
    publish two view packets to the renderer
    record the native scene for the left eye
    record the native scene for the right eye
    submit post effects that are safe for stereo
    release both swapchain images
    xrEndFrame with two projection layers

The initial implementation should use two eye images. An array texture or multiview path can be added after the two-eye path produces stable depth and correct IPD. Each eye needs its own view matrix and projection matrix.

## View packet

A render-view packet should contain:

    eye index
    view matrix
    projection matrix
    world camera position
    near and far clip
    viewport rectangle
    eye image handle
    frame sequence

The packet must be written with a sequence counter or another atomic handoff. The render thread should consume one complete packet for each eye.

The game camera remains the source of gameplay orientation. The headset pose contributes a calibrated local head transform. The patch must keep head pose, weapon aim, animation, and body pose as separate signals.

## Where the second view belongs

The first useful patch point is after the camera state is assembled and before the view and projection constants are uploaded. Static candidates are:

- `0x1432309C0` for camera source-derived state
- `0x14322FBD0` for the normalized camera output record
- the first render-graph function that consumes that output record
- the final constant-buffer upload discovered through frame capture

The renderer should execute the same scene passes twice with different camera constants. A shared depth buffer is unsafe when the two eye views diverge. Use a depth image per eye or a layered depth resource with correct view indexing.

## Post-processing policy

Screen-space effects need a classification.

- Per-eye effects can run after each eye scene.
- View-independent effects can run once before the eye split.
- Mono effects need a deliberate policy such as a world-space quad, an eye-locked layer, or a disabled feature.
- Mirror and portal passes must carry the eye index or be rendered into a separate mono surface.

The static `resolve_mirror_reprojection` path is a warning that some passes assume a single camera. It should not be reused as the native stereo path.

## Runtime milestones

### M0 presenter trace

Capture one frame with a desktop translation runtime. Record command queue creation, target resource creation, resource state transitions, the final submission, and the host present call.

### M1 dual image smoke test

Render a solid color into each OpenXR eye image. Confirm that the headset sees two independent images and that the window mirror remains stable.

### M2 per-eye world view

Use the original scene with per-eye view and projection constants. Validate IPD using a known-size object and a fixed world-space ruler.

### M3 native post effects

Re-enable post effects one group at a time. Log every effect that remains mono or uses the mirror resolve path.

### M4 input and body

Add OpenXR actions and the VRIK layer after stable stereo and depth exist.

## Failure conditions

Stop the stereo experiment when any of these occur.

- Only one eye receives a complete scene.
- Eye images are released before GPU completion.
- The translation runtime copies both eyes into one mono presenter.
- Culling removes geometry that exists in the other eye.
- Temporal or screen-space buffers are reused across eyes without per-eye history.

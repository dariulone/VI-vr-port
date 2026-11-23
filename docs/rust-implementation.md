# Rust implementation plan

The mod should be organized as a Rust workspace with a small unsafe boundary. Reverse-engineered addresses and calling conventions stay in one adapter crate. OpenXR frame logic, projection math, culling policy, pose solving, logging, and configuration remain safe Rust.

## Workspace layout

    gta6-vr-port/
      crates/
        vr-core/
        vr-openxr/
        vr-render-bridge/
        vr-pose/
        vr-input/
        vr-telemetry/
        vr-testkit/
      tools/
        ida/
      docs/

### vr-core

Owns frame identifiers, eye packets, world-scale calibration, configuration, diagnostics, and feature gates. It has no process-specific addresses and can be unit tested without a game.

### vr-openxr

Owns the OpenXR instance, session, spaces, swapchain images, action sets, view location, and frame timing. It publishes a complete two-eye packet through a lock-free or sequence-checked handoff.

The checked-in `gta6-vr-openxr` crate currently provides the safe frame coordinator and stereo-view handoff. The platform-specific OpenXR bindings can be connected at this boundary after the runtime presents a usable host process.

### vr-render-bridge

Owns the unsafe FFI and hook adapters for the translated renderer. It contains:

- image base and RVA resolution
- version and hash checks
- opaque Xbox device and queue handles
- camera packet injection
- PresentX or host-present tracing
- per-eye resource selection
- frame completion signals

No pose math belongs here.

### vr-pose

Owns skeleton calibration, pose records, two-bone arm IK, optional leg IK, and blend policies. It accepts opaque source pose records until the vector semantics are confirmed by a trace.

### vr-input

Maps OpenXR actions to the game's input bridge. Handedness and bindings are data-driven.

### vr-telemetry

Writes bounded logs for frame id, eye index, queue submission, camera constants, culling mode, pose count, and hook failures. It must never write game resources or raw buffers to the public repository.

### vr-testkit

Contains pure Rust tests for projection matrices, frustum unions, calibration, pose blending, and record-size checks.

## Unsafe boundary

Only these modules should contain unsafe code.

    vr-render-bridge/src/ffi.rs
    vr-render-bridge/src/hooks.rs
    vr-render-bridge/src/rvas.rs

Every FFI function should have a safe wrapper that validates:

- image base and module range
- function RVA
- pointer alignment
- record length
- eye index
- frame sequence
- resource state assumptions

The wrapper returns a typed error and disables the feature gate after a failed validation. It must not continue through an invalid pointer.

## Shared Rust data types

Use explicit C-compatible structures at the hook boundary.

The checked-in packet and manifest types use explicit `repr(C)` or integer enum representations. This makes the layout stable for the future native hook layer while the opaque pose payload remains byte-addressable.

`vr-core` exposes `RuntimeProbeFrame` with validation for the observed 256-record queue limit and 69,888-byte presentation payload.

    #[repr(C)]
    pub struct EyePacket {
        pub frame_id: u64,
        pub eye: u8,
        pub _pad: [u8; 7],
        pub view: [[f32; 4]; 4],
        pub projection: [[f32; 4]; 4],
        pub camera_world: [f32; 3],
        pub near_z: f32,
        pub far_z: f32,
        pub viewport: [u32; 4],
        pub image_handle: u64,
    }

The private pose boundary should initially use an opaque record.

    #[repr(C)]
    pub struct PoseRecord128 {
        pub bytes: [u8; 128],
    }

The current static evidence proves a 0x80-byte per-bone stride and a hash-to-record lookup. It does not prove the component order of all 128 bytes, so the Rust type must remain opaque until runtime capture validates the layout.

The pose crate also records the proven metadata byte offset `0x63` and the four auxiliary blocks. It exposes that byte for diagnostics while keeping the remaining record opaque.

## Frame state machine

Keep the state transitions explicit.

    Idle
      -> AcquiringViews
      -> RenderingLeft
      -> RenderingRight
      -> ReleasingImages
      -> Submitted
      -> Idle

An error state records the frame id and disables only the affected feature. OpenXR session loss, image-acquire failure, and invalid renderer pointers must not leave a swapchain image held.

## Native stereo bridge

The renderer adapter should expose these safe operations.

    begin_frame()
    set_eye_packet(EyePacket)
    begin_eye(Eye)
    end_eye(Eye)
    submit_frame()

The first implementation can call the original renderer once per eye. A later single-pass path can be added after the two-eye path proves depth, culling, and post effects.

The Rust bridge must not synthesize a mono image and reproject it. Each eye gets its own view, projection, depth, and temporal history policy.

## Render bridge targets

The first Rust adapter manifest should expose these static targets.

| Target | RVA | Rust wrapper role |
|---|---:|---|
| Camera output builder | 0x322F5D0 | Capture original camera output values |
| Camera output finalizer | 0x322F8D0 | Publish the four-component camera packet |
| Present lifecycle | 0x2484180 | Select the current presentation slot |
| Presentation core | 0x24826A0 | Submit plane resources and schedule frame events |
| Queue submission helper | 0x2482660 | Track command or synchronization completion |
| Pose copy | 0x523B680 | Copy and validate the 0x80-byte pose records |
| Bone hash mapper | 0x523C6F0 | Resolve hashes and build render-facing records |

The wrappers should log the frame id and resource ids around these calls. They should never assume that the presentation core has the same signature as a desktop IDXGISwapChain method.

The checked-in `gta6-vr-render-bridge` manifest mirrors these targets and also includes the camera view and clip updates, pose source and finalizer, skinning record builder, PrecisionAimContext, weapon component mapping, and projectile-location registration. This keeps the boot-day hook list in one versioned Rust constant while the TOML file remains convenient for external harnesses.

The camera render-object handoff is included as `camera_owner_tick`, `camera_render_prepare`, `camera_packet_copy`, `camera_render_object_init`, and `camera_render_object_accessor`. These targets let the first live trace start at the camera owner and stop at the render-owned packet before the generic resource submission helpers.

The bridge exposes the two worker queue slots as constants. `QUEUE_BATCH_SUBMIT_SLOT` is `0x118` for command or resource batches, and `QUEUE_PLANE_SUBMIT_SLOT` is `0x100` for presentation descriptors.

## Asymmetric projection

OpenXR supplies per-eye tangents. Keep the pure math in vr-core.

    pub struct EyeFov {
        pub left: f32,
        pub right: f32,
        pub up: f32,
        pub down: f32,
    }

    pub fn projection_from_tangents(
        fov: EyeFov,
        near_z: f32,
        far_z: f32,
    ) -> [[f32; 4]; 4]

The function must be tested against the engine's row or column convention after a constant-buffer capture. The hook layer only transports the resulting matrix.

## Pose and VRIK bridge

The intended Rust call order is:

    read source pose handle
    copy or lock the local-player pose record
    solve head and hand targets
    write parent-relative local transforms
    call the existing pose finalizer
    let the hash mapper and ped_skinning_cs continue

The static pose path is:

    0x14523B630 source pose lookup
    0x14523B680 pose copy
    0x14523BD00 pose finalization
    0x14523C6F0 hash-to-record mapping
    0x1427C15A0 skinning record construction

Rust should insert the solver between the pose copy and the finalizer. The first runtime version should modify only the local player and should preserve the original record when calibration or ownership checks fail.

## Versioning and test gates

The adapter should use a version manifest with:

- input SHA-256
- image base
- function RVAs
- expected prologue bytes
- pose stride
- supported renderer ABI

The mod should refuse to attach when the manifest does not match. Pure math tests can run on every build. FFI smoke tests require a local translated runtime and stay outside CI that has no game process.

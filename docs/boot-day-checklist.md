# Boot-day VR checklist

This is the shortest path from a confirmed translated boot to a native stereo smoke frame. The checklist assumes the executable is available locally and the translation runtime has a documented launch command.

## Before launch

- Record the translation-runtime commit and local binary hash.
- Build the Rust workspace in release mode.
- Verify that the OpenXR runtime is selected and a headset session can start.
- Keep one process instance only.
- Start the logger before the game process.
- Keep the IDA database closed to edits during the live trace.

## First 15 minutes

### Capture the original frame

Attach the Rust bridge at these static boundaries.

| Boundary | RVA | First data to record |
|---|---:|---|
| Device initialization | 0x2478440 | device pointer, HWND, resource arrays |
| Frame entry | 0x41E9E0 | frame id, context pointer |
| Worker | 0x2481E90 | record type, queue index, command count |
| Queue batch submit | 0x2481E90 | queue object, slot +0x118, batch count, flag 3 |
| Queue execute or sync | 0x2482660 | queue object, slot +0x118 completion arguments |
| Plane submission | 0x2481E90 | queue object, slot +0x100, presentation count, v86/v84 arrays |
| Present lifecycle | 0x2484180 | current four-slot index and token |
| Presentation core | 0x24826A0 | plane count, source resources, payload |
| Presentation completion | device vtable +0x288 | payload size 69888, returned record count, 56-byte completion records, events 9/10 |
| Camera render packet | 0x2008AE0 -> 0x3231B30 -> 0x53159B0 | owner pointer, three 16-byte destination blocks at render-object +0x30/+0x40/+0x50 |

The first goal is one stable mono frame through the original presentation path. Preserve the captured plane and token values.

### Solid-color stereo smoke test

1. Create one OpenXR projection layer with two eye swapchains.
2. Acquire both eye images before scene recording.
3. Submit a left-eye solid red image and a right-eye solid blue image through the native presentation bridge.
4. Verify that the headset sees different colors in each eye.
5. Release both images only after the queue completion signal.

This test proves the eye-resource contract before any camera or VRIK code is enabled.

## First native scene

Use the camera output path.

1. Capture the output vector written by 0x14322F8D0.
2. Locate the first constant-buffer upload that consumes the vector.
3. Replace the single view packet with left and right packets from xrLocateViews.
4. Build asymmetric projection matrices from each eye's four OpenXR tangents.
5. Keep gameplay FOV and headset projection as separate values.
6. Start with union-frustum instance and portal culling.
7. Add per-eye meshlet culling only after both eyes show complete geometry.

Do not change the weapon or body pose until the two-eye world view is stable.

## Pose and VRIK

The first local-player pose experiment uses this order.

    source pose from 0x523B630
    copy records at 0x523B680
    apply local-player VRIK to 0x80-byte records
    finalize at 0x523BD00
    resolve hashes at 0x523C6F0
    build skinning records at 0x27C15A0

Record the bone count, source pointer, hash table, and four vector blocks before changing values. Start with head and dominant hand only.

## Decoupled aim

Enable aim after VRIK has a stable hand pose.

1. Observe the trigger action and PrecisionAimContext at 0xF1B130.
2. Resolve the Muzzle component from the weapon-part path at 0x1D1BBF0.
3. Build a shot packet from controller grip, weapon root, and Muzzle transform.
4. Use Muzzle forward for the ray or the Wpn_LOC_Projectile launch direction.
5. Keep camera orientation independent from weapon aim.
6. Preserve the original reticle and assist policy behind a Rust feature flag.

The fire or damage resolver remains a live-trace target. Do not replace it until the launch record or hit resolver is identified.

## Evidence to save locally

- one mono frame trace
- one red/blue eye trace
- one camera constant-buffer capture
- one left/right culling comparison
- one source pose snapshot before and after a hand move
- one Muzzle and projectile launch snapshot

Keep traces and extracted buffers local. Commit only derived addresses, aggregate observations, and authored Rust code.

## Stop conditions

Pause the native stereo patch when:

- the two eyes show the same resource
- a queue or fence token is reused before completion
- one eye loses geometry because of culling
- temporal history is shared without an eye key
- pose records fail the 0x80-byte stride check
- a weapon shot still follows the camera after the Muzzle packet is valid

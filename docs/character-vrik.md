# Character pose and VRIK

## Static evidence

The executable exposes the following animation and character anchors.

| Anchor | Address | RVA | Meaning |
|---|---:|---:|---|
| `SKEL_PELVIS` | `0x146786C28` | `0x6786C28` | Character bone string |
| `SKEL_R_HAND` | `0x146787F2C` | `0x6787F2C` | Character bone string |
| `SKEL_L_HAND` | `0x146787F38` | `0x6787F38` | Character bone string |
| `GoalRotationOffsetsKeyedArray` | `0x14677891C` | `0x677891C` | Goal rotation data label |
| Goal rotation reader | `0x144391850` | `0x4391850` | Reads indexed goal rotation offsets |
| `MotionMatcherStore` | `0x146782158` | `0x6782158` | Motion matching store registration |
| `PoseMatcherStore` | `0x14678216B` | `0x678216B` | Pose matching store registration |
| Ped motion data | `0x146785DF7` | `0x6785DF7` | `common:/data/ai/PedMotionData` |
| Ped skinning setup | `0x1427C0400` | `0x27C0400` | Registers `ped_skinning_cs` |
| BVH skinning setup | `0x140CC5160` | `0xCC5160` | Registers `bvh_skinning` and `bvh_skinning_smh` |

The pose matcher setup contains a `DefaultPosematcher` resource path and computes pose-related transforms. The skinning setup registers the compute shader and fills descriptor tables for the final vertex deformation stage.

No string named `VRIK` or `FABRIK` was found. The existing character path must be retargeted through the game's own pose representation.

## Skeleton facts from the public format work

The measured humanoid resources use master skeletons with 596 and 560 named joints. The common names include:

- `SKEL_ROOT`
- `SKEL_Pelvis`
- `SKEL_Head`
- `SKEL_L_Hand`
- `SKEL_R_Hand`

The character skeleton format stores local rotation, local translation, scale, parent index, bone index, and inverse bind matrices. Mesh palettes map local bone hashes to global skeleton indices. Skin streams use four or eight influences with normalized weights.

These facts make a native pose override feasible after the pose buffer is located. They do not prove that every character uses the same master rig.

## VRIK insertion point

The intended order is:

    animation evaluation
    motion matching and pose matching
    local-player VRIK solve
    weapon and hand attachment update
    cloth and secondary motion
    ped_skinning_cs
    draw

The solve should operate on local parent-relative transforms. World-space device poses are converted into the game's calibrated avatar space before solving.

### Minimum solve

- Head target from the calibrated HMD pose
- Left and right hand targets from controller grip poses
- Two-bone arm solves with elbow hints
- Pelvis and chest stabilization from head and hand targets
- Optional leg solve from foot trackers or ground contacts
- Blend weight that returns control to gameplay animation during cutscenes, vehicles, ragdolls, and network ownership changes

### Calibration

Use a per-avatar calibration record.

    bone hash
    reference local rotation
    reference model position
    device offset
    arm length scale
    handedness
    world scale

The calibration must be recomputed when the player model changes. A fixed offset table is unsafe across the measured skeleton classes.

## Runtime validation still needed

The static pass has proven the pose handoff and the render-facing bone map. Runtime tracing is still needed to identify the producer of the source pose block and confirm the meaning of each 32-byte vector block. The next trace should watch:

1. The source pose object returned by `sub_14523B630`
2. The caller that produces the local-player pose before `sub_14523B680`
3. The four vector blocks returned by `sub_144DAAFC0`
4. The resource bound to `ped_skinning_cs`
5. The attachment update that consumes `SKEL_L_Hand` and `SKEL_R_Hand`

A successful trace should confirm the source producer, vector semantics, and ownership rules before enabling live VRIK writes.

## VRIK validation

- Check the pelvis, head, and both hands in a neutral calibration pose.
- Verify that arm lengths remain constant over a full reach.
- Verify weapon sockets follow the solved hand bones.
- Disable VRIK during a cinematic and verify that the original animation resumes.
- Compare the skinning buffer before and after one hand movement.

## Static evidence update

The pose handoff is now located. The per-ped render update is at 0x1435B3B30, with a second owner path at 0x1403ADF90. Both paths copy pose data and finalize it before the skinning resource builder.

The pose copy function at 0x14523B680 reads the count at source offset +0x0C, copies records from +0x10, and uses a 0x80-byte stride per bone. The hash mapper at 0x14523C6F0 reads one bone hash per index, resolves it through 0x144DAAFC0, and copies four 32-byte vector blocks into the render-facing arrays.

The recommended VRIK write boundary is the local parent-relative pose block immediately before 0x14523C6F0. This keeps the existing hash map, metadata finalizer, and ped_skinning_cs path intact.

## Auxiliary arrays and metadata validation

The finalizer has a second data path that matters for a safe VRIK write.

- `0x14523C690` invokes `0x14523C6F0` four times with source and destination offsets `0`, `+0x10`, `+0x20`, and `+0x30`, selecting auxiliary block indices `0` through `3`.
- Each `0x14523C6F0` call reads the pose count from `a2 + 0x0C`, resolves one 32-bit bone hash per index through `0x144DAAFC0`, and writes the resolved record data into a block-specific render array.
- When a pose object is present, `0x14523BD00` builds those arrays at `a1 + 0x880`, calls `0x14523C810`, and then copies an auxiliary validity value at `+0x0C` into the metadata record at `a1 + 0x300 + 0x63` for each bone. The metadata stride is `0x80` bytes.
- When the pose object is absent, `0x14523BD00` recomputes the same validity bytes with SIMD range checks. It does not synthesize a replacement pose.

The solver should therefore preserve the auxiliary-array and metadata path. A live trace should record the pose count, the four auxiliary destinations, the metadata byte at `+0x63`, and the hash-resolved source record before enabling a hand or head override.

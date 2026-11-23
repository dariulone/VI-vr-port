# Weapon aim and decoupled firing

## Static anchors

| Anchor | Address | RVA | Evidence |
|---|---:|---:|---|
| GameInput interface creation | 0x145F3FED0 | 0x5F3FED0 | Imported `GameInputCreate` call target |
| GameInput wrapper | 0x1459258A0 | 0x59258A0 | Creates or replaces the stored input interface |
| Weapon event dispatcher | 0x145F34810 | 0x5F34810 | Routes `(action_id, phase)` through the weapon handler table |
| Precision aim state | 0x140F1B130 | 0xF1B130 | Initializes PrecisionAimContext state at object offsets +1712..+1715 |
| Precision aim update | 0x144ACB720 | 0x4ACB720 | Reads weapon state and updates aim-related context |
| Aim context state walk | 0x144AC7B60 | 0x4AC7B60 | Walks weapon and aim state records, updates state flags |
| Weapon-part mapping | 0x141D1BBF0 | 0x1D1BBF0 | Resolves Mag, Muzzle, Accs, Barrel, Bolt, Stock, Sight, Grip, Trigger, and Under |
| Projectile location registration | 0x145B20870 | 0x5B20870 | Registers Wpn_LOC_Projectile in a weapon or projectile system |
| Reticle and aim UI | 0x141F69F10 | 0x1F69F10 | Sets reticle type, size, offsets, accuracy, aiming, reload, and weapon-ready state |
| First-person red dot | 0x1432B5C40 | 0x32B5C40 | Registers FPWeaponRedDot and its motion-blur mask |
| Projectile hit VFX | 0x14041BB90 | 0x41BB90 | Registers vfx_proj_decal_hit_detection, trail, and vbuffer resources |
| Missile event table | 0x1458FD140 | 0x58FD140 | Registers TARGETED_LOCK, ACQUIRING_TARGET, TARGET_ACQUIRED, and MISSLE_FIRED |

## Precision aim context

sub_140F1B130 lazily initializes a named PrecisionAimContext and stores a compact state record at object offsets +1712 through +1715. It has separate paths for weapon states and a fallback path that clears the context.

sub_144ACB720 updates that context from the weapon resource selected by the entity at a2 + 156. It calls sub_144AC7B60 to walk the weapon state record, checks state flags, and dispatches updates through sub_1437C9360 and related helpers. The context is a better hook target than the HUD reticle because it runs in gameplay state processing.

## Input and weapon event boundary

The only imported GameInput symbol is `GameInputCreate` at `0x145F3FED0`. The wrapper `0x1459258A0` lazily creates the interface, stores it in `qword_149AB15E8`, and calls its vtable initialization method. No named GameInput poll, trigger, button, or state symbol is imported. The click edge therefore enters through an interface vtable and cannot be identified from import names alone.

The first concrete gameplay boundary is the weapon event dispatcher `0x145F34810`. A pointer table at `0x1466CFD08` stores it as the first handler. The function receives `(a1, action_id, phase)` and routes the action family as follows.

| `action_id` | Phase 0 | Phase 1 | Phase 2 | Static meaning |
|---:|---|---|---|---|
| `0` | `0x144ACAB00` | `0x144ACAB80` | state flag | generic weapon update |
| `1` | `0x144ACABF0` | `0x144ACAD60` | `0x144ACB400` | weapon state and animation/resource transition |
| `2` | `0x144ACB540` | `0x144ACB720` | `0x144ACB940` | aim and precision context |
| `3` | `0x144ACB990` | `0x144ACBA50` | state flag | weapon readiness/aim gating candidate |
| `4` | `0x144ACBB90` | `0x144ACC050` | resource transition | weapon mode or equipment transition |
| `5` | `0x144ACC140` | `0x144ACC280` | auxiliary state | vector/resource handoff |
| `6` | `0x144ACC310` | `0x144ACC3B0` | `0x144ACC520` | active weapon selection/state |
| `7` | `0x144ACC580` | `0x144ACC6A0` | flag path | cleanup or auxiliary state |
| `8` | `0x144ACC6D0` | `0x144ACC710` | resource release | weapon resource state |
| `9` | `0x144ACC770` | `0x144ACC790` | `0x144ACC800` | final state validation |

The dispatcher is a state boundary, not a confirmed fire resolver. The phase-1 `action_id=2` path reaches `PrecisionAimContext`, while the phase-1 handlers for `action_id=1`, `3`, `4`, `5`, `6`, `7`, `8`, and `9` update weapon state, animation, resources, or readiness flags. Static analysis has not shown an ammo decrement, ray query, damage event, projectile allocation, or muzzle-velocity write in this dispatcher family.

## Muzzle and weapon sockets

sub_141D1BBF0 resolves weapon component names from a table.

    Mag
    Muzzle
    Accs
    Barrel
    Bolt
    Stock
    Sight
    Grip
    Trigger
    Under

These are weapon-part component references. They are separate from the character skeleton hashes. The weapon's muzzle transform should be read from the resolved Muzzle component or from its attachment record after the weapon pose is updated.

The character skeleton already exposes hand and prop attachment families. A decoupled aim implementation should drive the weapon root from the dominant grip and let the existing weapon-part map supply Muzzle and Sight transforms.

## Projectile and hit path

`0x145B20870` registers `Wpn_LOC_Projectile` in a resource or entity system. Its only code xref is from the large resource loader at `0x145C663F0`, where it is installed into a resource table. This proves an authored projectile socket or location family, but it is not a projectile spawn call and does not expose velocity or lifetime.

`0x1458FD140` builds an event table for missile and target states. `MISSLE_FIRED` and `TARGET_ACQUIRED` are aircraft or lock-on style event labels. They are not evidence that ordinary firearm shots use physical projectiles.

`0x14041BB90` initializes only `vfx_proj_decal_hit_detection`, `vfx_proj_decal_trail`, and `vfx_proj_decal_vbuffer`. It is called by the resource switch at `0x1456BDE30` and is a presentation registration path. It does not prove the hit resolver or ballistic simulation.

The image contains trigger metadata strings such as `TriggerType`, `TriggerSubType`, `willTrigger`, and `sfIsWeaponReady`, plus ammo UI fields. No direct gameplay string named `shoot`, `raycast`, `muzzle velocity`, or `projectile spawn` was found. The current static evidence therefore cannot classify the ordinary weapon path as hitscan or physical projectile.

### What can be concluded now

- A click must pass through the GameInput interface, then a weapon event or state dispatcher. The exact input-to-`action_id` mapping is unresolved.
- `PrecisionAimContext` and the weapon component map are state and pose preparation. They are upstream of a shot but do not themselves fire.
- `Wpn_LOC_Projectile` is a registered socket/resource family. It does not prove that every weapon spawns a projectile.
- Projectile-labelled VFX and the missile event table are presentation or specialized event resources. They do not prove the ordinary firearm resolver.
- Hitscan is not proven because no ray-query call with origin, direction, mask, and result structure was statically isolated.
- Physical projectile is not proven because no spawn record with position, direction or velocity, owner, and per-frame update was statically isolated.

## Decoupled aim design

The game trigger should remain the action that authorizes a shot. The aim direction should come from the controller and weapon pose.

    HMD pose
        -> view and camera only

    dominant grip pose
        -> weapon root orientation
        -> muzzle transform
        -> shot ray or projectile launch direction

The first implementation should use this packet.

    frame_id
    weapon_entity
    dominant_hand
    grip_pose_world
    offhand_pose_world
    muzzle_pose_world
    sight_pose_world
    ray_origin_world
    ray_direction_world
    trigger_pressed

For hitscan weapons, the ray origin should be the Muzzle transform and the direction should be the Muzzle forward axis after the grip-to-weapon correction. For physical projectiles, pass the same origin and direction into the projectile spawn or launch record associated with Wpn_LOC_Projectile.

The correction must preserve the weapon's authored hand offsets. A stable procedure is:

1. Read the weapon root and dominant grip transforms.
2. Compute the controller-to-weapon rotation from calibration.
3. Apply the rotation to the weapon root.
4. Recompute Muzzle and Sight from the weapon hierarchy.
5. Use the recomputed Muzzle forward vector for the shot.
6. Solve the offhand after the weapon root is updated.

Do not rotate the gameplay camera to follow the weapon. The view pose and weapon aim are independent signals.

## Assist and reticle policy

The HUD path exposes reticle size, X and Y offsets, accuracy, and accuracy range. Those values should remain a presentation layer.

- A world-space reticle follows the Muzzle ray and its hit distance.
- A head-locked reticle follows the sight picture and can be disabled when the physical sight is aligned.
- Aim assist can operate on the Muzzle ray after the raw direction is computed.
- A camera-center crosshair must not become the source of fire direction.

The original aim-assist policy should be preserved behind a Rust feature gate. The default VR mode should use raw Muzzle direction, then add an explicit assist cone only when the user enables it.

## Rust boundary

The Rust bridge should expose:

    read_weapon_components(entity)
    read_muzzle_pose(entity)
    update_weapon_root(entity, pose)
    build_shot_packet(aim_packet)
    submit_trigger_action(action)

Only the first four operations need render or gameplay hooks. The shot packet should be an opaque C-compatible record until the fire resolver is traced. The hook must validate weapon ownership and avoid writing remote or non-player entities.

## Runtime trace targets

1. `GameInputCreate` at `0x145F3FED0` and interface wrapper `0x1459258A0`. Record the returned object and its vtable calls for one trigger press and release.
2. Weapon dispatcher `0x145F34810`. Log `(action_id, phase)`, weapon entity, state flags at `a1+0xC4`, `a1+0xD0`, `a1+0xCA`, and resource pointer `a1+0x1D0`.
3. `PrecisionAimContext` update at `0x144ACB720` and aim state walk at `0x144AC7B60`.
4. Muzzle component resolution at `0x141D1BBF0`, followed by the resolved component transform.
5. Any new allocation or event after the trigger. A one-shot call carrying origin/direction and an immediate result indicates a hitscan candidate. A persistent object with owner, transform, velocity, and later collision callback indicates a projectile candidate.
6. Damage or hit application after the query or projectile collision. Correlate target entity, hit position, material, bone, and damage event.
7. `Wpn_LOC_Projectile` resource use and any write to its resolved transform or launch record.

This trace will distinguish hitscan, physical projectile, and event-driven weapon paths before any Rust hook is enabled.

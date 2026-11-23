# Weapon aim and decoupled firing

## Static anchors

| Anchor | Address | RVA | Evidence |
|---|---:|---:|---|
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

sub_145B20870 registers Wpn_LOC_Projectile in a resource or entity system. The exact projectile spawn caller is still stripped and needs a runtime trace from the trigger action.

sub_1458FD140 builds an event table for missile and target states. It is useful for event tracing because MISSLE_FIRED and TARGET_ACQUIRED are explicit event labels.

sub_14041BB90 initializes projectile hit VFX resources. It proves a hit-detection presentation path, but it does not prove the hit resolver or ballistic simulation.

No direct string named fire, shoot, raycast, or muzzle velocity was present. The exact hitscan or projectile damage resolver remains a runtime target.

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

1. GameInput trigger action to the weapon state update
2. PrecisionAimContext update at 0x144ACB720
3. Muzzle component resolution at 0x141D1BBF0
4. MISSLE_FIRED or projectile spawn event
5. Hit resolver and damage application
6. Wpn_LOC_Projectile transform and launch velocity

This trace will distinguish hitscan, physical projectile, and event-driven weapon paths before any Rust hook is enabled.

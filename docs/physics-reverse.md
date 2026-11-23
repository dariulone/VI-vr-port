# Physics reverse engineering

This document separates static physics-related evidence from render-only resources. The executable does not expose a Havok, PhysX, Bullet, or Chaos import. The physics core appears to be built from proprietary data-driven systems, GPU jobs, entity resources, and shared RAGE runtime helpers.

All physics anchors in this document are also present in the checked-in Rust RVA manifest for future version-gated tracing.

## Backend evidence

| Observation | Evidence | Interpretation |
|---|---|---|
| No named physics middleware import | Import search found no Havok, PhysX, Bullet, or Chaos module | The solver ABI cannot be identified from imports. Runtime object tracing is required. |
| GPU particle collision jobs | `0x141AF3BA0` registers `GpuParticleUpdate`, `GpuParticleCollision`, `GpuParticleEmit`, and `GpuParticleSort` | Particle collision is a GPU job family. It is separate from character and vehicle collision. |
| Xbox BVH helper | `XboxBvhBuilder.dll` import and `bvh_skinning` resources | Ray-tracing acceleration and skinned geometry support exist. This does not prove gameplay collision queries. |
| Constraint resource names | `0x1426870F0` parses names beginning with `constraint_` and returns the numeric suffix | Constraint records are data-driven. The function is a name/index resolver, not the solver. |

## Vehicle physics and deformation

### Vehicle request system

`0x145814350` registers `VehicleRequests` and `VehicleRequestsSimple` against the runtime resource system. It installs callbacks at `sub_140438B20`, `sub_1419EAD10`, `sub_1408B9760`, `sub_1432E7300`, and `sub_1432E7310`, then initializes the request subsystem.

This path is a streaming or entity-request layer. It proves vehicle-specific runtime resources, but it does not prove wheel contact, suspension, tire forces, or collision response.

### Vehicle deformation

`0x140CBF1F0` registers the `vehicle_deform` resource. The initializer creates buffers with allocation sizes `393216` and `98368`, clears global deform state, and initializes the resource tables used by the deform jobs.

The initializer derives the allocation stride from a runtime format value, allocates both resources with the same resource flags, obtains handles through `sub_1406EB740`, and clears a 32-entry table of 16-byte records. This is consistent with a buffered deform or damage staging system. The static function is initialization only. It does not receive a collision contact or write a vehicle pose.

The two follow-up setup calls are `sub_145B02F40` and `sub_145432060`. They initialize the per-deform resource layout with identity-like defaults before the global 32-slot table is reset. The table is a staging/index layer, not a wheel-contact representation.

The static evidence supports this path.

    vehicle collision or damage event
        -> vehicle deform resource update
        -> deform buffer or geometry record
        -> vehicle deformation render path

The collision impulse producer and the per-vehicle deformation update are still unresolved. A runtime trace should watch writes to the global resource handle created by `0x140CBF1F0`, then correlate one impact with the deform buffer and drawable update.

### Vehicle damage

`0x142511AB0` initializes two named accumulation buffers.

- `ExplosionVehicleDamageBuffer` uses a 6208-byte allocation.
- `ExplosionPedDamageBuffer` uses a 1312-byte allocation.

These buffers indicate an event accumulation stage for explosion damage. They do not establish the collision query, impulse integration, or final health application. The first useful live trace follows one explosion from event creation into the corresponding buffer write and then into damage application.

The buffers are zeroed during initialization, registered with the resource service, and given process-exit cleanup handlers. Their sizes and separate vehicle/ped identities make them useful runtime correlation points for the damage fan-out.

## Character bodies, ragdoll, and pose

No `ragdoll` literal was found. The character path is therefore identified through state and pose ownership rather than a named subsystem.

The verified handoff is:

    pose source lookup 0x523B630
        -> pose copy 0x523B680, 0x80-byte records
        -> pose finalizer 0x523BD00
        -> four auxiliary arrays 0x523C690 / 0x523C6F0
        -> hash-to-bone map
        -> ped_skinning_cs and BVH skinning

The finalizer gates updates through `0x523C810` and maintains a metadata byte at `+0x63` in each `0x80`-byte record. This is the safe VRIK insertion boundary. A ragdoll override must first identify the local-player state and ownership flags that select the pose producer.

Static facts do not prove a PhysX joint graph, capsule shape layout, contact material model, or ragdoll solver ABI. Those remain live-trace targets.

## Cloth and secondary motion

`PlClothes` and `PedClothes` strings exist. The strongest xref is `0x145CB8080`, which reads `PlClothes` while validating cinematic or vehicle metadata fields. This is a content and state gate. It does not expose cloth integration or constraint solving.

The public pose path places cloth and secondary motion after the local-player VRIK solve and before `ped_skinning_cs`. That ordering is an implementation design based on the existing pose handoff. The cloth solver and its particle buffers still need a runtime resource trace.

## Particle, rain, and environmental collision

### GPU particles

`0x141AF3BA0` registers four GPU jobs.

    GpuParticleUpdate
        -> GpuParticleCollision
        -> GpuParticleEmit
        -> GpuParticleSort

The job descriptors are indexed through a table initialized by `sub_1415DE1D0`. The static path confirms GPU particle collision as a render/compute subsystem. It does not prove body collision or gameplay hit reporting.

### Rain collision

`0x141D87EF0` constructs a large weather resource set and registers `raincollision`. It creates the persistent rain resource at `qword_14A02CF00` and connects it to weather and particle helpers. This is environmental particle collision and wet-surface interaction. It is separate from actor collision.

The named rain resource is created with a 3936-byte object, then passed through weather helpers before a larger 29632-byte persistent resource is attached to the weather owner. This indicates a small collision descriptor plus a longer-lived simulation or render state object.

The constructor responsible for the named descriptor is `0x145D44230`. It creates the resource with type code `41`, stores a small set of dimensions and flags, and initializes a 3920-byte state tail. The persistent weather object is created by `0x145C0C5F0` and attaches multiple helper arrays plus a `depthonly_draw` resource. The static path supports rain collision and environmental wetness resources, but it does not expose actor contact queries.

## Water simulation and interaction

`0x141E490F0` registers `water_compressed_simulation_rendertarget` and attaches settings entries to the water resource.

`0x1426205C0` switches water-system states. Its initialization state creates a water resource, copies three vector blocks into global water state, initializes several water resource handles, and registers `water_heightpass`, `water_depthoccluderpass`, and `water_mainscenedeferred` passes.

The state switch has separate branches for reset, resource creation, and pass registration. The water path therefore has explicit lifecycle phases. A VR port should keep the water simulation state shared and duplicate only eye-dependent water rendering passes.

`0x145B83070` maps the water mode field at `a2 + 444` to internal modes `1` through `5`, copies dimensions from `a2 + 412`, `+416`, and `+420`, and selects default vector/format values. `0x142620140` creates the water system's encoded resource name and state object. These functions define water configuration and lifecycle, not buoyancy or swimmer force integration.

The static evidence supports a GPU water simulation and rendering pipeline.

    water state update
        -> compressed simulation render target
        -> height pass and depth occluder
        -> main-scene deferred water pass

Buoyancy, swimmer contact, vehicle water forces, and wake impulses are not proven by these render targets. They require a trace from an actor entering water to the gameplay movement or force update.

## Constraint resources

`0x1426870F0` reads a runtime resource name, checks for the `constraint_` prefix, parses a two-digit suffix, and returns the numeric index. This is a registry lookup used by a higher-level constraint system. The function does not create joints or solve constraints.

The runtime trace should capture the resolved constraint index, owner entity, two connected bodies, rest transform, and solver update call before any VR hand or weapon constraint is changed.

## Horse collision response

`0x141D8C9B0` registers `Horse Collision Response`, builds a 34-byte response record, assigns a resource vtable, and passes the record to `sub_141279100`. This is a concrete species-specific collision-response data resource. It is not the general collision solver.

## Weapon collision and hit detection

The static weapon anchors are:

- `PrecisionAimContext` at `0x140F1B130` and update at `0x144ACB720`
- weapon component mapping at `0x141D1BBF0`
- `Muzzle` and `Wpn_LOC_Projectile`
- projectile hit VFX at `0x14041BB90`
- event table at `0x1458FD140`

No direct fire, raycast, damage, muzzle-velocity, or hit-resolver string was found. The exact gameplay collision path remains a runtime target.

## VR implications

- Native stereo changes view and culling inputs. Physics world state should remain shared between eyes.
- VRIK writes belong in local parent-relative pose records before pose finalization. They should not modify collision shapes until body ownership and ragdoll state are known.
- Decoupled weapon aim should feed the existing Muzzle or projectile launch path. Camera orientation must remain independent.
- Water, rain, and particle collision passes need an eye policy for their render outputs, while gameplay forces remain mono world state.
- Per-eye physics simulation would create divergent gameplay state and is outside the native stereo design.

## Runtime gaps

1. Identify the physics world owner and tick boundary.
2. Identify the collision query API and contact result structure.
3. Trace one vehicle impact through `vehicle_deform` and damage application.
4. Trace one ped ragdoll activation into the pose producer and constraint system.
5. Trace one projectile from trigger authorization to hit or damage resolution.
6. Trace one water entry to buoyancy or movement force update.
7. Capture cloth and particle buffers only after their ownership and update cadence are known.

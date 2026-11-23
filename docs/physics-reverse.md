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

The string search also found no gameplay-facing `contact`, `broadphase`, `solver`, or `ragdoll` names. The remaining collision-labelled strings are the rain, GPU-particle, horse-response, and unrelated platform or audio paths listed below. This makes call-chain and object-state tracing more important than name-based discovery for the unresolved solver boundary.

## Direct physics parameter parsing

Two functions expose the clearest rigid-body and contact parameters found in the image.

### Contact and spring material parameters

`0x144896800` reads optional fields from a physics material/config object and stores them in a compact runtime record.

| Config key | Destination offset |
|---|---:|
| `friction` | `+0x24` |
| `elasticity` | `+0x28` |
| `springconstant` | `+0x30` |
| `springdamping` | `+0x34` |
| `rollingfriction` | `+0x38` |

The parser asks the source object for each value through its vtable and leaves the defaults intact when a key is absent. This is a material/contact parameter record. It does not itself integrate impulses or solve contacts.

### Rigid-body mass, inertia, gravity, and buoyancy

`0x144454BE0` reads rigid-body properties and writes a body-state record.

| Config key | Runtime operation |
|---|---|
| `mass` or `density` | Computes the body mass and stores it at `+0x4C` |
| `angInertia` | Reads a vector-like value and combines it with the body state |
| `inertiaBox` | Converts box inertia into a vector record |
| `inertiaScale` | Scales the inertia vector |
| `gravityFactor` | Stores the scalar at `+0x60` |
| `buoyancyFactor` | Stores the scalar at `+0x6C` |

The parser calls `sub_1453808B0` and dispatches a body update through a vtable method at byte offset `+0x70`. These calls are the strongest static handoff into a rigid-body backend. The actual solver tick, broadphase, contact manifold, and integration step remain unresolved.

### Vehicle handling data

`0x1456D8090` loads `common:/data/ai/DefaultVehicleAiHandlingEntry`. This is a vehicle handling resource path. It should be traced alongside the rigid-body record when investigating vehicle force generation, but it does not prove that handling values are the same structure as physical mass or contact material data.

### Damping and force coefficients

`0x144455000` wraps the rigid-body parser and additionally reads `damping`, `linearC`, `linearV`, `linearV2`, `angularC`, `angularV`, and `angularV2`. The vector coefficients are copied into body fields at `+0x70`, `+0x80`, `+0x90`, `+0xA0`, `+0xB0`, and `+0xC0`. This is the clearest static schema for translational and angular damping/force coefficients.

`0x141F97990` is a dynamic update candidate. When the body type at the resolved body object equals `4`, it sets a body-state flag, calls `0x1453808B0` on the body state, passes a 56-byte temporary record through `0x14517E960`, and returns through `0x144EE47B0`. These are high-priority runtime breakpoints for identifying the solver update and body integration cadence.

### Body-shape batch and state preparation

The caller at `0x140645740` gives a more detailed static view of the body update boundary.

- The input batch stores a count at `a1 + 0x0C` and an entry array at `a1 + 0x10`.
- Each entry exposes a range through 16-bit fields at `+0x20` and `+0x22`. Shape records are walked with an 80-byte stride and use a shape or material index at record offset `+0x44`.
- The routine writes the owner context into the body at `+0x48`, sets entry state at `+0x54`, sets body flags at `+0x18`, `+0x19`, and `+0x154`, and filters shapes through virtual methods at byte offsets `+0x50` and `+0x58`.
- For active shapes it expands source vectors into 64-byte records, generates normalized four-component orientation or parameter vectors, and writes per-shape scalar metadata. The normalization uses reciprocal-square-root instructions, which is consistent with quaternion or normalized basis preparation.
- `0x140F22840` reduces a count of source elements, computes a reciprocal of the aggregate scalar, and scales the corresponding scalar and vector arrays. `0x144F5B760` is the small vector-scale helper used by this stage.
- `0x144F5B820` accumulates weighted vector values and normalizes the resulting vector. `0x144F5B8C0` accumulates weighted second-moment terms from offsets around an aggregate center. This is an inertia or shape-mass preparation candidate, not proof of the contact solver itself.
- The prepared arrays pass through `0x144F5B770` and then into `0x141F97990`. This creates a concrete static chain from shape records to body-state update.

The compact helper chain is therefore:

    shape batch 0x140645740
        -> aggregate and second-moment preparation 0x144F5B770
        -> body-state update 0x141F97990
        -> normalized state 0x14517E960 / 0x14517E9F0

The body-state helpers expose additional fields. `0x1453808B0` writes mass at `+0x4C` and recomputes inverse mass at `+0x5C`. `0x14517E960` clamps and merges an input vector into `+0x40`, normalizes the derived vector at `+0x50`, stores a 16-bit state or index at `+0x60`, and updates state flags through `0x14517E9D0`. `0x144EE47B0` merges another vector into the same `+0x50` record. This identifies state normalization and flag propagation around the update, while the integration, broadphase, and contact impulse stages remain unlocated.

### Center of mass and inertia preparation

`0x142687A20` is a second body-property path with a more explicit shape aggregate. It reads a body-owned shape array through `a1 + 0x170`, takes the element count from the body record at `+0x164`, and builds one 64-byte transformed record per element. The input vectors are converted into a basis-like block, optional per-shape coefficients at `+0x50`, `+0x54`, and `+0x58` are applied, and the largest active coefficient is tracked.

The aggregate sequence is:

1. `0x14521B7F0` obtains the source shape data through the object's virtual method at byte offset `+0x570` and produces source vectors plus scalar weights.
2. `0x1450B75E0` expands the shape records into transformed per-element blocks.
3. `0x1455B1160` accumulates `weight * vector` and divides by the total weight. Its output is written into the body vector at `+0x50`, making it a weighted-center or center-of-mass candidate.
4. `0x1455B1200` subtracts that center from each transformed vector and accumulates weighted quadratic terms. This is the strongest static inertia or second-moment candidate in the image.
5. `0x1453808B0` recomputes inverse mass, the body vtable method at byte offset `+0x70` receives the resulting inertia vector, and `0x14517E960` normalizes the state and stores the selected shape index at `+0x60`.

The same helper pair is used by `0x1406D9FF0`, which initializes a body from its shape count, calls both aggregate routines, updates the body inertia, and sums buoyancy-like coefficients from shape records at `+0x6C`. This is a static initialization and property-update path. It still does not expose the broadphase, contact manifold, impulse solve, or time integration.

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

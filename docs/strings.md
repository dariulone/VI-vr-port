# Curated technical strings

This is a small, purpose-selected list of renderer, camera, culling, skinning, and character labels. It is a navigation aid for independent static analysis. It is not a dump of the executable's full string table.

| String | Address | RVA | Category | Cross-reference |
|---|---:|---:|---|---:|
| `sgaWindowClass` | `0x14677B672` | `0x677B672` | window and renderer init | `0x142478440` |
| `projectionCenter` | `0x14677D8F0` | `0x677D8F0` | projection data label | config table |
| `perspectiveProjection` | `0x14677EFE2` | `0x677EFE2` | projection data label | config table |
| `PerspectiveProjection` | `0x14677EFF8` | `0x677EFF8` | projection data label | config table |
| `fieldOfView` | `0x146778FCD` | `0x6778FCD` | camera data label | config table |
| `CameraSetting` | `0x146780E78` | `0x6780E78` | settings group | `0x145A44800`, `0x14324A230` |
| `Camera_Move` | `0x14678187F` | `0x678187F` | camera event | `0x142650D40` |
| `Eye Presets` | `0x14677B3D6` | `0x677B3D6` | eye configuration | `0x1456B0B70` |
| `DynStereo%d` | `0x146785006` | `0x6785006` | stereo-named object family | `0x141F8BEC0` |
| `InstanceCulling` | `0x146781173` | `0x6781173` | culling subsystem | `0x145700770` |
| `meshlet_culling` | `0x146781144` | `0x6781144` | culling subsystem | `0x145B03CB0` |
| `portal_prepass_stencil_culling` | `0x146781154` | `0x6781154` | portal visibility | `0x140CCC430`, `0x14374F390` |
| `resolve_mirror_reprojection` | `0x14677EFC6` | `0x677EFC6` | mirror and reprojection path | `0x140CCC430`, `0x14374F390` |
| `ped_skinning_cs` | `0x14677C8FC` | `0x677C8FC` | skinning compute shader | `0x1427C0400` |
| `bvh_skinning` | `0x1467810F3` | `0x67810F3` | BVH skinning | `0x140CC5160` |
| `bvh_skinning_smh` | `0x146780C78` | `0x6780C78` | BVH skinning | `0x140CC5160` |
| `SKEL_PELVIS` | `0x146786C28` | `0x6786C28` | character bone | string table |
| `SKEL_R_HAND` | `0x146787F2C` | `0x6787F2C` | character bone | string table |
| `SKEL_L_HAND` | `0x146787F38` | `0x6787F38` | character bone | string table |
| `GoalRotationOffsetsKeyedArray` | `0x14677891C` | `0x677891C` | pose goal data | `0x144391850` |
| `MotionMatcherStore` | `0x146782158` | `0x6782158` | motion matching | `0x145B40DF0` |
| `PoseMatcherStore` | `0x14678216B` | `0x678216B` | pose matching | `0x145B92B00` |
| `common:/data/ai/PedMotionData` | `0x146785DF7` | `0x6785DF7` | ped motion metadata | `0x14598AAE0` |
| `PrecisionAimContext` | `0x1467798EB` | `0x67798EB` | weapon aim context | `0x140F1B130`, `0x144ACB720` |
| `Muzzle` | `0x146782BE8` | `0x6782BE8` | weapon component | `0x141D1BBF0` |
| `Wpn_LOC_Projectile` | `0x146782DC4` | `0x6782DC4` | projectile location | `0x145B20870` |
| `sfIsAiming` | `0x146781117` | `0x6781117` | reticle and aim UI | `0x141F69F10` |
| `MISSLE_FIRED` | `0x146788028` | `0x6788028` | missile event | `0x1458FD140` |
| `VehicleRequests` | `0x14677B1D3` | `0x67B1D3` | vehicle request system | `0x145814350` |
| `VehicleRequestsSimple` | `0x146782D1A` | `0x682D1A` | vehicle request system | `0x145814350` |
| `vehicle_deform` | `0x14677FA3E` | `0x677FA3E` | vehicle deformation | `0x140CBF1F0` |
| `ExplosionVehicleDamageBuffer` | `0x14677E0A8` | `0x677E0A8` | vehicle damage accumulation | `0x142511AB0` |
| `ExplosionPedDamageBuffer` | `0x14677E0C5` | `0x677E0C5` | ped damage accumulation | `0x142511AB0` |
| `GpuParticleCollision` | `0x14677F497` | `0x677F497` | GPU particle collision | `0x141AF3BA0` |
| `raincollision` | `0x14677F489` | `0x677F489` | weather particle collision | `0x141D87EF0` |
| `PlClothes` | `0x14677C4B9` | `0x677C4B9` | clothes state gate | `0x145CB8080` |
| `water_compressed_simulation_rendertarget` | `0x14677AA5C` | `0x677AA5C` | water simulation target | `0x141E490F0` |
| `water_heightpass` | `0x14677B5BE` | `0x677B5BE` | water height pass | `0x1426205C0` |
| `water_depthoccluderpass` | `0x14677B5CF` | `0x677B5CF` | water depth pass | `0x1426205C0` |
| `constraint_` | `0x146786059` | `0x6786059` | constraint index naming | `0x1426870F0` |
| `Horse Collision Response` | `0x146781E46` | `0x6781E46` | species collision response | `0x141D8C9B0` |

## Import labels

- `D3D12XboxCreateDevice`
- `D3D12SerializeVersionedRootSignature`
- `D3D12ConfigureVirtualMemoryX`
- `GameInputCreate`
- `XboxBvhBuilder`
- `ffxCreateContext`
- `ffxDispatch`
- `XMemVirtualAlloc`

No OpenXR, OpenVR, SteamVR, or `xr*` runtime entry point was found in the import table.

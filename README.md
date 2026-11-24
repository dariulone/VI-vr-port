# VI VR Port Research

Static reverse-engineering notes and an implementation plan for a native OpenXR stereo port.

The repository contains derived facts, RVAs, curated technical strings, and authored integration designs. It does not contain the executable, game resources, keys, full decompiled functions, or extracted package data.

## Current status

- IDA Pro 9.2 database created for the analyzed x64 image
- Native graphics initialization located
- Camera state and view-update candidates located
- Camera render packet, visibility, frustum, and culling boundaries mapped
- Instance, meshlet, portal-stencil, and skinning paths located
- OpenXR, OpenVR, and SteamVR imports or strings were not found
- Native stereo presenter and runtime tracing remain to be implemented
- Exact CPU pose write site remains to be identified with a running translated build

## Reading order

1. [Static analysis](docs/static-analysis.md)
2. [Render reverse engineering](docs/render-reverse.md)
3. [Native stereo](docs/native-stereo.md)
4. [Projection and culling](docs/projection-and-culling.md)
5. [Character pose and VRIK](docs/character-vrik.md)
6. [Physics reverse engineering](docs/physics-reverse.md)
7. [Curated strings](docs/strings.md)
8. [Research status](docs/research-status.md)
9. [Public references](docs/references.md)
10. [External project status](docs/external-status.md)
11. [Rust implementation](docs/rust-implementation.md)
12. [Weapon aim](docs/weapon-aim.md)
13. [Boot-day checklist](docs/boot-day-checklist.md)
14. [Rust RVA manifest](docs/rust-rva-manifest.toml)
15. [IDA static harness](docs/ida-harness.md)
16. [Runtime probe schema](docs/runtime-probe-schema.md)
17. [x64dbg live attach map](docs/x64dbg-live-attach.md)
18. [Completion matrix](docs/completion-matrix.md)
19. [Rust workspace](crates/vr-core/src/lib.rs)

## Analysis identity

| Field | Value |
|---|---|
| Analysis date | 2026-11-23 |
| IDA | 9.2 |
| Image base | `0x140000000` |
| Input SHA-256 | `E95020B2FA4A721DB3DF59FDECF75621730AC559D8835252F6C3B658D5A08993` |
| Entry point | `0x1456A1EC8` |
| Function count | 434,530 |

The hash is included so independent researchers can confirm that their local analysis refers to the same executable. The file itself is never required by this repository.

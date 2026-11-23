# GTA6 VR Port Research

Static reverse-engineering notes and an implementation plan for a native OpenXR stereo port.

The repository contains derived facts, RVAs, curated technical strings, and authored integration designs. It does not contain the executable, game resources, keys, full decompiled functions, or extracted package data.

## Current status

- IDA Pro 9.2 database created at `work/ida/game_scarlett_master.i64`
- Native graphics initialization located
- Camera state and view-update candidates located
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
6. [Curated strings](docs/strings.md)
7. [Research status](docs/research-status.md)
8. [Public references](docs/references.md)
9. [External project status](docs/external-status.md)
10. [Rust implementation](docs/rust-implementation.md)
11. [Weapon aim](docs/weapon-aim.md)
12. [Boot-day checklist](docs/boot-day-checklist.md)
13. [Rust RVA manifest](docs/rust-rva-manifest.toml)

## Analysis identity

| Field | Value |
|---|---|
| Analysis date | 2026-11-23 |
| IDA | 9.2 |
| Image base | `0x140000000` |
| Input SHA-256 | `E95020B2FA4A721DB3DF59FDECF75621730AC559D8835252F6C3B658D5A08993` |
| Entry point | `0x1456A1EC8` |
| Function count | 434,406 |

The hash is included so independent researchers can confirm that their local analysis refers to the same executable. The file itself is never required by this repository.

# Changelog

All notable changes to **Lutetian** will be documented in this file.

## [0.2.0](https://github.com/hamides-dev/lutetian/compare/v0.1.0...v0.2.0) (2026-11-20)


### Features

* add interactive CAD tools, embedded architectural samples, lightbox, skill installer, and cross-platform fixes ([ad11b5f](https://github.com/hamides-dev/lutetian/commit/ad11b5f04f122a9b3a12a2611f95f2da2488485f))
* initial Lutetian Studio workspace, scale validator, MCP bridge, and Antigravity plugin ([63100a8](https://github.com/hamides-dev/lutetian/commit/63100a861c91c68079ff7fce5f3dd7fee93964f3))


### Bug Fixes

* **app:** gate app_icon field for cross-platform clippy cleanly ([c572376](https://github.com/hamides-dev/lutetian/commit/c572376026e6e892f499218ea09c7a807015cfe6))

## [0.1.0] - 2026-11-20

### Features

- **100% Pure-Rust Native Studio (`iced 0.14`):** Zero WebView overhead with Midnight Dark / Daylight Light Velvet Plum (`#685C74`) UI parity.
- **Stage-Gated Pipeline (`lutetian-core`):** Enforces Stage 1 (`Project Bible & Scale Anchors`), Stage 2 (`5-Angle Multi-View Completion`), Stage 3 (`Master Blueprint CAD`), Stage 4 (`Deduplicated Asset BOM`), Stage 5 (`Deterministic Math Scale Validator`), and Stage 6 (`Engine Sync Export`).
- **Interactive Multi-Layer 2D CAD Blueprint Canvas:** Pan/zoom metric grid (`1 unit = 1m`), rulers, 5 toggleable layers (*Zoning & Surface*, *Elevation & Height*, *Footprint & Metrics*, *Navigation & Scale* with draggable `1.8m` human & `4.5m` vehicle silhouettes, and *Asset ID Map*), plus live dimension inspector and violation highlighting.
- **Live Agent Mission Control & Compliance Dashboard:** Real-time heartbeat, 3-column Execution Pipeline Board (`Completed`, `In Progress`, `Upcoming`), Compliance Matrix (`COMPLIANT`, `PENDING`, `VIOLATED`) with click-to-inspect entity zoom, and live activity stream.
- **Localhost Bridge & Antigravity MCP Plugin (`lutetian-bridge`, `lutetian-mcp`, `antigravity-plugin/`):** Bidirectional HTTP JSON-RPC + WebSocket bridge, stdio MCP server, 5 modular `SKILL.md` guides, and one-command setup scripts.

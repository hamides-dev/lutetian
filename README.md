# Lutetian — Game Environment Architecture Studio

**Lutetian** is an ultra-lightweight, 100% pure-Rust desktop application (`iced 0.14` + `wgpu`/`tiny-skia`, zero WebView) for **Game Environment Architecture & Pre-Production**. It serves as the Architectural Brain, Stage-Gated Orchestrator, Deterministic Math Scale Validator, and Live Agent Mission Control guiding AI agents (starting with **Google Antigravity**) from a single concept image to verified multi-view concepts, metric-accurate 2D multi-layer blueprints, and dimension-locked modular asset catalogs.

---

## Key Capabilities

- **Zero WebView / Ultra-Lightweight Native Rust**: Built on `iced 0.14` with Midnight Dark / Daylight Light Velvet Plum (`#685C74`) styling, bundled `Inter` & `Vazirmatn` fonts, and Material Symbols Rounded vector iconography.
- **Stage-Gated Architecture Pipeline (Stages 1–6)**:
  1. **Stage 1 — Ingestion & Metric Scale Anchors (`Project Bible`)**: Locks reference art style, lighting, and human/door/floor/road scale anchors (`human_height = 1.8m`, `door_height = 2.1m`, `floor_height = 3.2m`, `road_lane_width = 3.5m`).
  2. **Stage 2 — Multi-View Completion**: Per-angle review gallery (`front`, `back`, `left`, `right`, `bird_eye`) with per-angle approval & regeneration requests.
  3. **Stage 3 — Master Blueprint & Multi-Layer 2D CAD**: Interactive pan/zoom canvas on a meter grid with live rulers and 5 toggleable layers (*Zoning & Surface*, *Elevation & Height*, *Footprint & Metrics*, *Navigation & Scale* with draggable `1.8m` human & `4.5m` vehicle silhouettes, and *Asset ID Map*).
  4. **Stage 4 — Deduplicated Asset Breakdown (BOM)**: Unique `AssetSpec` entries with locked `width_m × depth_m × height_m` bounding boxes and isolated reference slots.
  5. **Stage 5 — Deterministic Pure-Rust Scale Validator**: Non-AI math engine enforcing building-to-human ratios, door clearances, floor heights, non-overlapping colliders, asset bbox parity, and world bounds.
  6. **Stage 6 — Engine Sync Export**: Exports `lutetian_manifest.json`, `master_blueprint.json`, and `asset_catalog_bom.json` for Blender / Unity / Unreal pipelines.
- **Live Agent Mission Control**: Real-time heartbeat, 3-state Execution Pipeline Board (`Completed` / `In Progress` / `Upcoming`), Compliance & Rules Matrix (`COMPLIANT` / `PENDING` / `VIOLATED`) with click-to-inspect entity focus, and live MCP tool stream.
- **Google Antigravity Plugin (`antigravity-plugin/`)**: Ships `lutetian-mcp` stdio server + 5 modular `SKILL.md` guides + one-command installers (`install.ps1` / `install.sh`).

---

## Building from Source

```powershell
# Run checks and unit/integration tests
cargo test --workspace

# Build optimized release binaries (lutetian + lutetian-mcp)
cargo build --release

# Launch Lutetian Studio
cargo run -p lutetian-app
```

---

## Connecting Google Antigravity

See [antigravity-plugin/README.md](antigravity-plugin/README.md) or run:

```powershell
.\antigravity-plugin\install.ps1
```

---

## License

MIT License — see [LICENSE](LICENSE) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

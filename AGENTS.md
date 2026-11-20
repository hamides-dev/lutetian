# AGENTS.md — Lutetian (Game Environment Architecture Studio)

## 1. Project Identity & Mission
- **Name:** `Lutetian`
- **Repository:** `https://github.com/hamides-dev/lutetian`
- **Purpose:** An ultra-lightweight, native desktop application in **Rust** for **Game Environment Architecture & Pre-Production**. Lutetian is NOT a 3D mesh modeling tool (it does not replace Blender). It is the **Architectural Brain, Stage-Gated Orchestrator, Deterministic Scale Validator, and Live Agent Mission Control** that guides AI agents (starting with Google Antigravity) from a single reference image to verified multi-view concepts, metric-accurate 2D multi-layer blueprints, and dimension-locked modular asset catalogs ready for Blender/Game Engines.

## 2. Hard Technical Constraints (Non-Negotiable)
1. **100% Native Rust — ZERO WebView:**
   - Strictly NO WebView, NO Tauri WebView, NO Electron, NO embedded Chromium/CEF, NO HTML/CSS/JS runtime.
   - Use a lightweight, pure-Rust native GPU-accelerated GUI framework (e.g., `eframe`/`egui` or `iced` — inspect `E:\langhian` first to see if it already uses a pure-Rust GUI crate; if `E:\langhian` uses a WebView, replicate its **exact visual design system** natively in pure Rust without WebView).
2. **Ultra-Lightweight Footprint:**
   - Fast cold start, minimal idle RAM and CPU usage, small release binary size (`opt-level = "z"` or `"s"`, LTO enabled, stripped symbols in release profile).
3. **Reference Project Mirroring (`E:\langhian`):**
   - **MANDATORY FIRST STEP:** Read and inspect `E:\langhian` before writing any application code.
   - Mirror `E:\langhian`'s exact visual UI language (color palette, typography, spacing, sidebar/panel hierarchy, window chrome, icon style, flat minimal aesthetic).
   - Mirror `E:\langhian`'s exact **Windows installer packaging**, **auto-updater mechanism**, versioning scheme, and **GitHub Releases / Actions CI pipeline**.
4. **GitHub Publication:**
   - Publish and configure repository under account **`hamides-dev`** (`hamides-dev/lutetian`) with working build, release, installer, and auto-update workflows matching `E:\langhian`.

## 3. Core Architecture (Workspace Layout)
Organize the project as a clean Cargo workspace:
- `crates/lutetian-core`: Domain models, Stage State Machine, Master Blueprint JSON schema, Scale Ruleset, Deterministic Math Validator, Project persistence.
- `crates/lutetian-app`: Pure-Rust native GUI (`Lutetian` desktop app), native 2D CAD-like multi-layer Blueprint renderer, Live Agent Telemetry & Compliance Dashboard, Auto-updater integration.
- `crates/lutetian-bridge`: Local IPC / WebSocket + HTTP JSON-RPC server running inside `lutetian-app` for real-time bidirectional sync with external agents and plugins.
- `crates/lutetian-mcp`: Standalone lightweight Rust MCP (Model Context Protocol) server binary that bridges **Google Antigravity** to `lutetian-app` via `lutetian-bridge`.
- `antigravity-plugin/`: Complete Google Antigravity plugin bundle containing MCP config, installation scripts, and stage-specific `SKILL.md` files.

## 4. Stage-Gated Environment Architecture Pipeline
Lutetian enforces strict phase transitions. An agent may NOT skip stages or mutate locked artifacts without user approval.

1. **Stage 1 — Ingestion & Scale Anchor (`Project Bible`):**
   - User provides 1+ reference images.
   - Agent extracts art style, mood, lighting, and locks **Metric Scale Anchors** (defaults editable by user: `human_height = 1.8m`, `door_height = 2.1m`, `floor_height = 3.2m`, `road_lane_width = 3.5m`).
2. **Stage 2 — Multi-View Completion:**
   - If the reference is single-angle, the agent generates/submits missing angles (`front`, `back`, `left`, `right`, `bird_eye`).
   - Each angle is displayed in Lutetian for per-angle user approval or regeneration request.
3. **Stage 3 — Master Blueprint & Multi-Layer 2D Engineering Maps:**
   - **Critical Rule:** Never rely on image-generation models to draw measurements on maps. The agent outputs structured metric **Master Blueprint JSON** on a meter-based coordinate grid.
   - `lutetian-app` natively renders exact, crisp, toggleable 2D map layers with real numbers and rulers:
     - **Footprint & Metrics Layer** (exact walls, footprints, X/Y lengths in meters)
     - **Elevation & Height Layer** (terrain contour, building heights on Z axis)
     - **Navigation & Scale Layer** (walkable paths, bottlenecks, draggable 1.8m human & vehicle scale silhouettes)
     - **Zoning & Surface Layer** (road, grass, dirt, water, residential/commercial zones)
     - **Asset ID Map Layer** (clickable element IDs e.g. `HOUSE_A_01` linked to the Asset Catalog)
   - Users must be able to pan/zoom the 2D canvas and inspect exact dimensions interactively.
4. **Stage 4 — Asset Breakdown & Isolation (Bill of Materials):**
   - Deduplicate repeating objects (e.g., 1 unique `TREE_PINE_01` referenced 14 times on the blueprint).
   - Every unique asset MUST have an explicit metric bounding box (`width_m`, `depth_m`, `height_m`) and an isolated reference image slot.
5. **Stage 5 — Deterministic Scale Validator (Pure Rust Math Engine):**
   - Non-AI mathematical validator executing automatically whenever Blueprint or Asset data changes.
   - Enforces hard constraints (e.g., `building.height_m >= human_height * 1.8`, `door.height_m > human_height`, `footprint_overlap == false`, `asset_bbox_matches_blueprint_footprint`).
   - Returns structured violation reports (Rule ID, Entity ID, Expected vs. Actual meters, Severity) both to the UI and back to the agent via MCP.
6. **Stage 6 — External Engine / Plugin Sync:**
   - Clean export/plugin API (JSON Manifest + Blueprint + Scale Rules + Asset Specs) ready for future Blender / Unity sync plugins.

## 5. Live Agent Telemetry & Compliance Dashboard (Core UI Requirement)
The user MUST have 100% real-time visibility into the connected agent at all times. Build a dedicated, always-accessible Mission Control panel / view inside Lutetian showing:
- **Live Activity Stream:** Exactly what the agent is doing *right now* (active phase, active tool/skill, current action message, live heartbeat/connection status).
- **Execution Pipeline Board (3 explicit states):**
  - **Completed:** Finished tasks/sections with timestamps and linked outputs.
  - **In Progress:** Currently executing task(s) with progress percentage / sub-steps.
  - **Upcoming (Planned):** Queued future tasks and stages not yet started.
- **Compliance & Rules Matrix:**
  - Every architectural requirement, stage gate, and scale rule listed explicitly with live status badges: `COMPLIANT`, `PENDING`, `VIOLATED`.
  - Clicking a violated or pending rule highlights the exact offending object on the 2D blueprint or asset list.

## 6. Mandatory Real-Time Agent Reporting Contract
Any agent connected via MCP is contractually required to report live state updates:
- Before starting any task: call `report_task_started(task_id, stage, title, plan_queue)`.
- During work: call `report_progress(task_id, status_message, percent)`.
- On completion: call `report_task_completed(task_id, summary, artifact_refs)`.
- On every rule/constraint check: call `report_compliance_update(rule_id, status, evidence)`.
- Expose `get_project_state`, `get_active_stage_skill`, `submit_angle_image`, `submit_blueprint_json`, `submit_asset_spec`, and `run_scale_validation` over MCP.

## 7. Google Antigravity Plugin Deliverable (`antigravity-plugin/`)
You MUST build a complete, ready-to-install plugin package for **Google Antigravity**:
1. **MCP Server (`lutetian-mcp`):** Compiled lightweight Rust binary communicating over stdio with Antigravity and forwarding live calls/events to `lutetian-app` over localhost WebSocket/HTTP.
2. **Modular `SKILL.md` Files:**
   - `skills/lutetian-live-protocol/SKILL.md`: Mandatory live telemetry & compliance reporting rules.
   - `skills/lutetian-stage1-analysis/SKILL.md`: Reference image analysis & metric scale anchor locking.
   - `skills/lutetian-stage2-multiview/SKILL.md`: Multi-angle completion & user approval loop.
   - `skills/lutetian-stage3-blueprint/SKILL.md`: Metric Master Blueprint JSON drafting rules.
   - `skills/lutetian-stage4-assets/SKILL.md`: Bill-of-Materials deduplication, bounding-box locking, and isolated asset sheet generation.
3. **Antigravity Config & Installer:** Ready-to-use MCP configuration JSON and a one-command setup script + `README.md` explaining how to attach the plugin to Antigravity.

## 8. Engineering & Delivery Workflow
1. **Inspect First:** Analyze `E:\langhian` thoroughly (UI layout, theme tokens, crate choices, installer scripts, updater implementation, `.github/workflows/`). Document findings in `docs/LANGHIAN_PARITY.md` before coding.
2. **Incremental Milestones:** Build and verify compile + tests at every milestone:
   - Milestone 1: Workspace setup, `lutetian-core` data models + Deterministic Scale Validator + unit tests.
   - Milestone 2: `lutetian-app` native Rust UI matching `E:\langhian` styling + Live Telemetry/Compliance Dashboard + interactive 2D Multi-Layer Blueprint Canvas.
   - Milestone 3: `lutetian-bridge` + `lutetian-mcp` + full `antigravity-plugin/` (`SKILL.md` pack + config + end-to-end live telemetry test).
   - Milestone 4: Installer packaging, auto-updater integration, GitHub Actions CI/CD, and push to `hamides-dev/lutetian`.
3. **Verification Gate:** Run `cargo check --workspace`, `cargo test --workspace`, and `cargo build --release` before marking any milestone complete. Never leave stubs, mock-only dead buttons, or TODO placeholders in core workflows.
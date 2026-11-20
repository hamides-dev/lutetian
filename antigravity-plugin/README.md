# Lutetian — Google Antigravity Plugin

This plugin connects **Google Antigravity** to **Lutetian Studio** over the Model Context Protocol (`lutetian-mcp`) and enforces live mission-control telemetry, stage gates, and metric scale validation across every environment pre-production step.

---

## Included Components

1. **`lutetian-mcp` (Stdio MCP Server)** — Lightweight native Rust binary forwarding MCP tool calls to `lutetian-app`'s local HTTP/WebSocket bridge (`127.0.0.1:47821`, auto-discovered via `bridge.json`).
2. **Modular `SKILL.md` Pack (`skills/`):**
   - `lutetian-live-protocol`: Mandatory live progress, task queue, and compliance reporting rules.
   - `lutetian-stage1-analysis`: Reference image ingestion & metric `ScaleAnchors` locking.
   - `lutetian-stage2-multiview`: 5-angle (`front`, `back`, `left`, `right`, `bird_eye`) completion & user approval loop.
   - `lutetian-stage3-blueprint`: Metric `MasterBlueprint` JSON drafting on a 1.0m coordinate grid.
   - `lutetian-stage4-assets`: Deduplicated Bill-of-Materials, locked bounding boxes, and Stage 6 Engine Sync export.
3. **One-Command Installers:** `install.ps1` (Windows) and `install.sh` (macOS / Linux).

---

## Quick Setup (Windows)

```powershell
# Run from repository root or installed Lutetian directory:
.\antigravity-plugin\install.ps1
```

## Quick Setup (macOS / Linux)

```bash
chmod +x ./antigravity-plugin/install.sh
./antigravity-plugin/install.sh
```

Both installers:
1. Build/install `lutetian-mcp` into your local binary directory.
2. Copy all 5 `lutetian-*` skills into `~/.gemini/antigravity/skills/`.
3. Emit `antigravity-plugin/mcp_config.resolved.json` with the exact `command` path to merge into Antigravity's `~/.gemini/config/mcp_config.json` (or click **Settings -> Bridge & Plugin -> Copy MCP Config** inside the Lutetian app!).

---

## Live Agent Workflow

1. Launch **Lutetian** (`lutetian.exe`). Notice the titlebar badge: `Bridge :47821`.
2. In Antigravity, ask the agent to build or architect a game environment using the Lutetian skills.
3. As the agent calls `report_task_started`, `report_progress`, `submit_project_bible`, `submit_angle_image`, `submit_blueprint_json`, `submit_asset_spec`, and `run_scale_validation`, Lutetian's **Mission Control**, **2D CAD Canvas**, and **Compliance Matrix** update instantaneously in real time.

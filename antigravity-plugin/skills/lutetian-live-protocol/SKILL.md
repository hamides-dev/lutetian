---
name: lutetian-live-protocol
description: Mandatory real-time telemetry, stage-gate discipline, and compliance reporting contract for any AI agent connected to Lutetian Studio via MCP.
---

# Lutetian Live Telemetry & Compliance Protocol

You are connected to **Lutetian Studio** via `lutetian-mcp`. Every action you perform MUST be visible to the user in Lutetian's **Mission Control & Compliance Dashboard** in real time.

## Non-Negotiable Reporting Contract

1. **Always Discover Active Stage First:**
   - Call `get_active_stage_skill` and `get_project_state` at session start to confirm the active unlocked stage and locked metric scale anchors.
2. **Before Starting Any Task:**
   - Call `report_task_started(task_id, stage, title, plan_queue)` where `plan_queue` lists the upcoming tasks so the **Upcoming (Planned)** column populates immediately.
3. **During Work (Every Step / Sub-Task):**
   - Call `report_progress(task_id, status_message, percent)` so the user sees live progress and current action text.
4. **On Completion:**
   - Call `report_task_completed(task_id, summary, artifact_refs)` listing any created/modified file paths or blueprint IDs.
5. **On Every Rule or Gate Verification:**
   - Call `report_compliance_update(rule_id, status, evidence)` with `status` in `COMPLIANT | PENDING | VIOLATED`.
   - Always keep `AGENT-001` marked `COMPLIANT` while actively reporting.

## Stage-Gate Enforcement Rules

- **Never skip stages.** Lutetian's Rust state machine will reject downstream writes until upstream stages are **Approved & Locked** by the user.
- **Never mutate locked artifacts.** If a stage is locked (`ApprovedLocked`), ask the user to unlock it in Lutetian or explain why a revision is requested.
- **Wait for user approvals.** After submitting Stage 1 (`submit_project_bible`) or Stage 2 (`submit_angle_image`), call `report_progress(..., "Awaiting user approval in Lutetian UI", 95)` and pause or poll cleanly until the user locks the stage.

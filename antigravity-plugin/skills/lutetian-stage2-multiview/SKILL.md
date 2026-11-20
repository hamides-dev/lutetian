---
name: lutetian-stage2-multiview
description: Stage 2 multi-view completion — generate and submit missing front, back, left, right, and bird_eye views via submit_angle_image and respect per-angle user approvals.
---

# Lutetian Stage 2 — Multi-View Completion & Approval Loop

Requires Stage 1 (`Project Bible`) to be `ApprovedLocked`. Always follow `lutetian-live-protocol`.

## Required Views

All five angles must be completed and approved in Lutetian before Stage 3 unlocks:
- `front`
- `back`
- `left`
- `right`
- `bird_eye`

## Workflow

1. Call `get_project_state()` to inspect which angles are still `pending` or `regenerate_requested`, and read user feedback notes on any rejected angles.
2. Call `report_task_started("stage2-multiview", 2, "Generating missing environment concept angles", ["Submit angles to Lutetian", "Await per-angle user approvals"])`.
3. For each missing or `regenerate_requested` angle:
   - Keep lighting, architectural proportions, and palette locked to Stage 1 `Project Bible`.
   - Save the rendered/generated image to the project's `multiview/` folder.
   - Call `submit_angle_image(angle, image_path, notes)`.
   - Call `report_progress("stage2-multiview", "Submitted <angle> for user review", pct)`.
4. Once all 5 angles are submitted, call `report_task_completed("stage2-multiview", "All 5 multi-view angles submitted; awaiting user per-angle approval", [...])`.
5. Do NOT proceed to Stage 3 until `get_active_stage_skill()` reports Stage 3 (`lutetian-stage3-blueprint`) is unlocked.

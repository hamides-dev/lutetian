---
name: lutetian-stage1-analysis
description: Stage 1 environment ingestion — analyze reference images, extract Project Bible art/lighting/palette specifications, and lock metric ScaleAnchors via submit_project_bible.
---

# Lutetian Stage 1 — Ingestion & Metric Scale Anchors

Always follow `lutetian-live-protocol` while executing Stage 1.

## Workflow

1. Call `report_task_started("stage1-bible", 1, "Analyzing reference images and calibrating metric scale anchors", ["Submit Project Bible", "Await user Stage 1 lock", "Stage 2 Multi-View completion"])`.
2. Inspect the user's supplied reference image(s) carefully:
   - Extract **art_style** (rendering paradigm, architectural era, surface fidelity).
   - Extract **mood** and **lighting** (sun elevation, key/fill contrast, sky/ambient hue).
   - Extract **palette_notes** (primary stone/wood/foliage/accent material hex tones).
3. Establish **Metric Scale Anchors** (`scale_anchors`):
   - Default human reference: `human_height_m = 1.8`
   - Default doorway clearance: `door_height_m = 2.1`, `door_width_m = 1.0`
   - Default floor-to-floor height: `floor_height_m = 3.2`
   - Default vehicular road lane: `road_lane_width_m = 3.5`
   - Default vehicle reference: `vehicle_length_m = 4.5`, `vehicle_width_m = 1.9`, `vehicle_height_m = 1.55`
   - Never allow `door_height_m <= human_height_m` or `floor_height_m < human_height_m * 1.4`.
4. Call `submit_project_bible(...)` with the extracted metadata, image file paths (`reference_images`), and `scale_anchors`.
5. Call `report_compliance_update("SCALE-001", "COMPLIANT", "Submitted verified metric scale anchors")` and `report_task_completed("stage1-bible", "Project Bible ready for user approval", reference_images)`.
6. Ask the user to review and click **Approve & Lock Stage 1** in Lutetian before advancing to Stage 2.

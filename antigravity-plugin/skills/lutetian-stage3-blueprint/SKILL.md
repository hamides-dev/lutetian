---
name: lutetian-stage3-blueprint
description: Stage 3 Master Blueprint authoring — produce structured metric MasterBlueprint JSON on a 1.0m coordinate grid (never image-drawn measurements) and verify with run_scale_validation.
---

# Lutetian Stage 3 — Metric Master Blueprint JSON

Requires Stage 2 (`Multi-View`) to be `ApprovedLocked`. Always follow `lutetian-live-protocol`.

## Critical Architectural Rule

**NEVER rely on image-generation models to draw measurements or floorplan dimensions.**
You MUST author structured metric **`MasterBlueprint` JSON** on a meter-based coordinate grid (`grid_unit_m = 1.0`), which Lutetian renders natively with exact rulers, silhouettes, and layers.

## MasterBlueprint Schema Contract

- `grid_unit_m`: `1.0`
- `world_width_m`, `world_depth_m`: Positive world dimensions in meters.
- `zones`: Array of `ZoneRegion` (`id`, `label`, `kind`: `grass|road|dirt|water|residential|commercial|plaza`, `rect: { x_m, y_m, width_m, depth_m, rotation_deg }`, `surface_material`).
- `elevations`: Array of `ElevationContour` (`id`, `label`, `rect`, `elevation_m`).
- `footprints`: Array of `FootprintEntity` (`id`, `asset_id`, `label`, `kind`: `building|wall|prop|foliage|vehicle|bridge|other`, `rect`, `height_m`, `base_elevation_m`, `floor_count`, `collidable`).
- `doors`: Array of `DoorOpening` (`id`, `parent_entity_id`, `label`, `position_m: { x_m, y_m }`, `width_m`, `height_m`, `sill_elevation_m`).
- `navigation_paths`: Array of `NavigationPath` (`id`, `label`, `points_m: [{ x_m, y_m }]`, `width_m`, `is_vehicular`).
- `scale_markers`: Include at least one `human` (`1.8m`) and `vehicle` (`4.5m`) marker for visual reference.
- `placements`: Additional `AssetPlacement` instances referencing deduplicated `asset_id`s.

## Hard Scale Constraints (Enforced by Pure-Rust Math Validator)

- Every `building` must have `height_m >= human_height_m * 1.8` AND `height_m >= floor_count * floor_height_m * 0.85`.
- Every `door` must have `height_m >= human_height_m + 0.15` and `width_m >= 0.85`.
- Collidable footprints MUST NOT intersect in 3D space (`SCALE-005`).
- All footprints MUST stay inside `[0..world_width_m, 0..world_depth_m]` (`SCALE-007`).
- Pedestrian `navigation_paths` need `width_m >= 1.2`, vehicular paths `width_m >= road_lane_width_m` (`SCALE-008`).

## Workflow

1. Call `report_task_started("stage3-blueprint", 3, "Drafting metric Master Blueprint JSON", ["Submit MasterBlueprint JSON", "Run Deterministic Scale Validator"])`.
2. Call `submit_blueprint_json(blueprint)`.
3. Immediately call `run_scale_validation()`. If any `ScaleViolation` is returned, fix the exact coordinates/dimensions and re-submit until `error_count == 0`.
4. Report compliance for `SCALE-002`..`SCALE-008` via `report_compliance_update`, then call `report_task_completed`.

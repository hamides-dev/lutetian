---
name: lutetian-stage4-assets
description: Stage 4 Asset Breakdown & BOM deduplication — register unique AssetSpecs with locked metric bounding boxes matching MasterBlueprint footprints and export Stage 6 Engine Sync bundle.
---

# Lutetian Stage 4 — Deduplicated Bill of Materials & Engine Sync

Requires Stage 3 (`Master Blueprint`) to be `ApprovedLocked`. Always follow `lutetian-live-protocol`.

## Workflow

1. Call `get_project_state()` and collect every unique `asset_id` referenced in `blueprint.footprints` and `blueprint.placements`.
2. Call `report_task_started("stage4-assets", 4, "Deduplicating repeating assets and locking metric bounding boxes", ["Register AssetSpecs", "Attach isolated reference images", "Run scale validation", "Export Stage 6 bundle"])`.
3. **Deduplication Rule:** Never create duplicate `AssetSpec` entries for repeated items (e.g., 1 unique `TREE_PINE_01` referenced across 14 placements/footprints).
4. For each unique `asset_id`, call `submit_asset_spec`:
   - `bbox_m`: `{ width_m, depth_m, height_m }` MUST match every corresponding `FootprintEntity` in the Master Blueprint within `±0.05m` (`SCALE-006`).
   - `isolated_image_path`: Provide a dedicated isolated asset reference image path (`GATE-004`).
   - `pivot`, `category`, `material_tags`, `notes`.
5. Call `run_scale_validation()` and confirm `passed == true` (`error_count == 0`, `warning_count == 0`).
6. Once the user locks Stage 4 (or when ready for Stage 6), call `export_engine_bundle(output_dir)` to emit `lutetian_manifest.json`, `master_blueprint.json`, and `asset_catalog_bom.json` for Blender / Unity / Unreal import.
7. Call `report_task_completed("stage4-assets", "Asset Catalog locked & Engine Sync manifest exported", [...])`.

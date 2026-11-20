use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::assets::{AssetSpec, BomSummaryEntry, build_bill_of_materials};
use crate::blueprint::MasterBlueprint;
use crate::project::{LutetianProject, ScaleAnchors};
use crate::validator::{ValidationReport, validate_project};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineSyncManifest {
    pub schema_version: String,
    pub exported_at: String,
    pub project_id: String,
    pub project_name: String,
    pub scale_anchors: ScaleAnchors,
    pub blueprint: MasterBlueprint,
    pub assets: Vec<AssetSpec>,
    pub validation: ValidationReport,
}

pub fn build_engine_sync_manifest(project: &LutetianProject) -> EngineSyncManifest {
    EngineSyncManifest {
        schema_version: "lutetian.engine_sync.v1".to_string(),
        exported_at: Utc::now().to_rfc3339(),
        project_id: project.id.clone(),
        project_name: project.name.clone(),
        scale_anchors: project.bible.scale_anchors,
        blueprint: project.blueprint.clone(),
        assets: project.assets.clone(),
        validation: validate_project(project),
    }
}

pub fn export_engine_bundle(
    project: &LutetianProject,
    output_dir: &Path,
) -> Result<PathBuf, std::io::Error> {
    fs::create_dir_all(output_dir)?;
    let manifest = build_engine_sync_manifest(project);
    let manifest_path = output_dir.join("lutetian_manifest.json");
    let json = serde_json::to_string_pretty(&manifest).map_err(std::io::Error::other)?;
    fs::write(&manifest_path, json)?;

    let bp_path = output_dir.join("master_blueprint.json");
    let bp_json =
        serde_json::to_string_pretty(&project.blueprint).map_err(std::io::Error::other)?;
    fs::write(bp_path, bp_json)?;

    let bom: Vec<BomSummaryEntry> = build_bill_of_materials(&project.assets, &project.blueprint);
    let bom_summary: Vec<serde_json::Value> = bom
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "asset_id": entry.asset.asset_id,
                "name": entry.asset.name,
                "category": entry.asset.category,
                "bbox_m": entry.asset.bbox_m,
                "pivot": entry.asset.pivot,
                "isolated_image_path": entry.asset.isolated_image_path,
                "instance_count": entry.instance_count,
                "entity_ids": entry.entity_ids,
            })
        })
        .collect();
    let bom_path = output_dir.join("asset_catalog_bom.json");
    let bom_json = serde_json::to_string_pretty(&bom_summary).map_err(std::io::Error::other)?;
    fs::write(bom_path, bom_json)?;

    Ok(manifest_path)
}

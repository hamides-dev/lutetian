use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::blueprint::MasterBlueprint;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MetricDimensions {
    pub width_m: f64,
    pub depth_m: f64,
    pub height_m: f64,
}

impl MetricDimensions {
    pub fn new(width_m: f64, depth_m: f64, height_m: f64) -> Self {
        Self {
            width_m,
            depth_m,
            height_m,
        }
    }

    pub fn is_positive(&self) -> bool {
        self.width_m > 0.0 && self.depth_m > 0.0 && self.height_m > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    #[default]
    Architecture,
    Foliage,
    Prop,
    StreetFurniture,
    TerrainFeature,
    Vehicle,
}

impl AssetCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Architecture => "Architecture",
            Self::Foliage => "Foliage",
            Self::Prop => "Prop",
            Self::StreetFurniture => "Street Furniture",
            Self::TerrainFeature => "Terrain Feature",
            Self::Vehicle => "Vehicle",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetSpec {
    pub asset_id: String,
    pub name: String,
    pub category: AssetCategory,
    pub bbox_m: MetricDimensions,
    #[serde(default = "default_pivot")]
    pub pivot: String,
    #[serde(default)]
    pub isolated_image_path: Option<String>,
    #[serde(default)]
    pub material_tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

fn default_pivot() -> String {
    "bottom_center".to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub struct BomSummaryEntry {
    pub asset: AssetSpec,
    pub instance_count: usize,
    pub entity_ids: Vec<String>,
}

/// Deduplicates blueprint references into a Bill of Materials indexed by unique `asset_id`.
pub fn build_bill_of_materials(
    assets: &[AssetSpec],
    blueprint: &MasterBlueprint,
) -> Vec<BomSummaryEntry> {
    let mut counts: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for fp in &blueprint.footprints {
        counts
            .entry(fp.asset_id.clone())
            .or_default()
            .push(fp.id.clone());
    }
    for pl in &blueprint.placements {
        let list = counts.entry(pl.asset_id.clone()).or_default();
        if !list.contains(&pl.instance_id) {
            list.push(pl.instance_id.clone());
        }
    }

    assets
        .iter()
        .map(|asset| {
            let entity_ids = counts.remove(&asset.asset_id).unwrap_or_default();
            BomSummaryEntry {
                asset: asset.clone(),
                instance_count: entity_ids.len(),
                entity_ids,
            }
        })
        .collect()
}

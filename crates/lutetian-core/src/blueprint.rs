use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintLayer {
    FootprintMetrics,
    ElevationHeight,
    NavigationScale,
    ZoningSurface,
    AssetIdMap,
}

impl BlueprintLayer {
    pub fn all() -> &'static [BlueprintLayer] {
        &[
            BlueprintLayer::ZoningSurface,
            BlueprintLayer::ElevationHeight,
            BlueprintLayer::FootprintMetrics,
            BlueprintLayer::NavigationScale,
            BlueprintLayer::AssetIdMap,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::FootprintMetrics => "Footprint & Metrics",
            Self::ElevationHeight => "Elevation & Height",
            Self::NavigationScale => "Navigation & Scale",
            Self::ZoningSurface => "Zoning & Surface",
            Self::AssetIdMap => "Asset ID Map",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct MetricPoint2D {
    pub x_m: f64,
    pub y_m: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MetricRect2D {
    /// Top-left / origin X coordinate in meters (before rotation around center or corner).
    /// Lutetian treats `(x_m, y_m)` as the minimum corner in local grid coordinates.
    pub x_m: f64,
    pub y_m: f64,
    pub width_m: f64,
    pub depth_m: f64,
    #[serde(default)]
    pub rotation_deg: f64,
}

impl MetricRect2D {
    pub fn new(x_m: f64, y_m: f64, width_m: f64, depth_m: f64) -> Self {
        Self {
            x_m,
            y_m,
            width_m,
            depth_m,
            rotation_deg: 0.0,
        }
    }

    pub fn center(&self) -> MetricPoint2D {
        MetricPoint2D {
            x_m: self.x_m + self.width_m * 0.5,
            y_m: self.y_m + self.depth_m * 0.5,
        }
    }

    /// Computes exact axis-aligned bounding box in world meters `(min_x, min_y, max_x, max_y)`
    /// accounting for rotation around rectangle center.
    pub fn world_aabb(&self) -> (f64, f64, f64, f64) {
        if self.rotation_deg.abs() < 1e-4 {
            return (
                self.x_m,
                self.y_m,
                self.x_m + self.width_m,
                self.y_m + self.depth_m,
            );
        }
        let c = self.center();
        let hw = self.width_m * 0.5;
        let hd = self.depth_m * 0.5;
        let rad = self.rotation_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let corners = [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)];
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for (lx, ly) in corners {
            let wx = c.x_m + lx * cos - ly * sin;
            let wy = c.y_m + lx * sin + ly * cos;
            min_x = min_x.min(wx);
            min_y = min_y.min(wy);
            max_x = max_x.max(wx);
            max_y = max_y.max(wy);
        }
        (min_x, min_y, max_x, max_y)
    }

    pub fn intersects_xy(&self, other: &MetricRect2D, tolerance_m: f64) -> bool {
        let (ax0, ay0, ax1, ay1) = self.world_aabb();
        let (bx0, by0, bx1, by1) = other.world_aabb();
        (ax0 + tolerance_m) < bx1
            && (ax1 - tolerance_m) > bx0
            && (ay0 + tolerance_m) < by1
            && (ay1 - tolerance_m) > by0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FootprintKind {
    #[default]
    Building,
    Wall,
    Prop,
    Foliage,
    Vehicle,
    Bridge,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FootprintEntity {
    pub id: String,
    pub asset_id: String,
    pub label: String,
    pub kind: FootprintKind,
    pub rect: MetricRect2D,
    pub height_m: f64,
    #[serde(default)]
    pub base_elevation_m: f64,
    #[serde(default = "default_one")]
    pub floor_count: u32,
    #[serde(default = "default_true")]
    pub collidable: bool,
}

fn default_one() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DoorOpening {
    pub id: String,
    pub parent_entity_id: String,
    pub label: String,
    pub position_m: MetricPoint2D,
    pub width_m: f64,
    pub height_m: f64,
    #[serde(default)]
    pub sill_elevation_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElevationContour {
    pub id: String,
    pub label: String,
    pub rect: MetricRect2D,
    pub elevation_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NavigationPath {
    pub id: String,
    pub label: String,
    pub points_m: Vec<MetricPoint2D>,
    pub width_m: f64,
    #[serde(default)]
    pub is_vehicular: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMarkerKind {
    Human,
    Vehicle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleMarker {
    pub id: String,
    pub kind: ScaleMarkerKind,
    pub x_m: f64,
    pub y_m: f64,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ZoneSurfaceKind {
    #[default]
    Grass,
    Road,
    Dirt,
    Water,
    Residential,
    Commercial,
    Plaza,
}

impl ZoneSurfaceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Road => "Road",
            Self::Dirt => "Dirt",
            Self::Water => "Water",
            Self::Residential => "Residential",
            Self::Commercial => "Commercial",
            Self::Plaza => "Plaza",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZoneRegion {
    pub id: String,
    pub label: String,
    pub kind: ZoneSurfaceKind,
    pub rect: MetricRect2D,
    #[serde(default)]
    pub surface_material: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetPlacement {
    pub instance_id: String,
    pub asset_id: String,
    pub x_m: f64,
    pub y_m: f64,
    #[serde(default)]
    pub z_m: f64,
    #[serde(default)]
    pub rotation_deg: f64,
    #[serde(default = "default_unit_scale")]
    pub scale_xyz: [f64; 3],
}

fn default_unit_scale() -> [f64; 3] {
    [1.0, 1.0, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MasterBlueprint {
    pub grid_unit_m: f64,
    pub world_width_m: f64,
    pub world_depth_m: f64,
    pub origin_label: String,
    pub footprints: Vec<FootprintEntity>,
    pub doors: Vec<DoorOpening>,
    pub elevations: Vec<ElevationContour>,
    pub navigation_paths: Vec<NavigationPath>,
    pub scale_markers: Vec<ScaleMarker>,
    pub zones: Vec<ZoneRegion>,
    pub placements: Vec<AssetPlacement>,
}

impl Default for MasterBlueprint {
    fn default() -> Self {
        Self {
            grid_unit_m: 1.0,
            world_width_m: 40.0,
            world_depth_m: 32.0,
            origin_label: "SW Corner (0m, 0m)".to_string(),
            footprints: Vec::new(),
            doors: Vec::new(),
            elevations: Vec::new(),
            navigation_paths: Vec::new(),
            scale_markers: vec![
                ScaleMarker {
                    id: "SCALE_HUMAN_01".to_string(),
                    kind: ScaleMarkerKind::Human,
                    x_m: 16.0,
                    y_m: 16.0,
                    label: "1.8m Human".to_string(),
                },
                ScaleMarker {
                    id: "SCALE_VEHICLE_01".to_string(),
                    kind: ScaleMarkerKind::Vehicle,
                    x_m: 20.0,
                    y_m: 24.5,
                    label: "4.5m Vehicle".to_string(),
                },
            ],
            zones: Vec::new(),
            placements: Vec::new(),
        }
    }
}

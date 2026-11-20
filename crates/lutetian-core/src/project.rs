use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use uuid::Uuid;

use crate::assets::{AssetCategory, AssetSpec, MetricDimensions};
use crate::blueprint::{
    AssetPlacement, DoorOpening, ElevationContour, FootprintEntity, FootprintKind, MasterBlueprint,
    MetricPoint2D, MetricRect2D, NavigationPath, ScaleMarker, ScaleMarkerKind, ZoneRegion,
    ZoneSurfaceKind,
};
use crate::telemetry::{
    ActivityLevel, AgentTelemetryState, ComplianceStatus, ExecutionTask, PipelineStage, TaskState,
};
use crate::validator::{ValidationReport, validate_project};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageGateStatus {
    #[default]
    NotStarted,
    InProgress,
    AwaitingApproval,
    ApprovedLocked,
}

impl StageGateStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Not Started",
            Self::InProgress => "In Progress",
            Self::AwaitingApproval => "Awaiting User Approval",
            Self::ApprovedLocked => "Approved & Locked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScaleAnchors {
    pub human_height_m: f64,
    pub door_height_m: f64,
    pub door_width_m: f64,
    pub floor_height_m: f64,
    pub road_lane_width_m: f64,
    pub vehicle_length_m: f64,
    pub vehicle_width_m: f64,
    pub vehicle_height_m: f64,
}

impl Default for ScaleAnchors {
    fn default() -> Self {
        Self {
            human_height_m: 1.8,
            door_height_m: 2.1,
            door_width_m: 1.0,
            floor_height_m: 3.2,
            road_lane_width_m: 3.5,
            vehicle_length_m: 4.5,
            vehicle_width_m: 1.9,
            vehicle_height_m: 1.55,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProjectBible {
    pub title: String,
    pub art_style: String,
    pub mood: String,
    pub lighting: String,
    pub palette_notes: String,
    pub reference_images: Vec<String>,
    pub scale_anchors: ScaleAnchors,
    pub locked: bool,
    pub updated_at: String,
}

impl Default for ProjectBible {
    fn default() -> Self {
        Self {
            title: "Untitled Environment".to_string(),
            art_style: String::new(),
            mood: String::new(),
            lighting: String::new(),
            palette_notes: String::new(),
            reference_images: Vec::new(),
            scale_anchors: ScaleAnchors::default(),
            locked: false,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngleKind {
    Front,
    Back,
    Left,
    Right,
    BirdEye,
}

impl AngleKind {
    pub fn all() -> &'static [AngleKind] {
        &[
            AngleKind::Front,
            AngleKind::Back,
            AngleKind::Left,
            AngleKind::Right,
            AngleKind::BirdEye,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Front => "Front Elevation",
            Self::Back => "Back Elevation",
            Self::Left => "Left Side",
            Self::Right => "Right Side",
            Self::BirdEye => "Bird's-Eye Overview",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Back => "back",
            Self::Left => "left",
            Self::Right => "right",
            Self::BirdEye => "bird_eye",
        }
    }

    pub fn from_key(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "front" => Some(Self::Front),
            "back" | "rear" => Some(Self::Back),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "bird_eye" | "birdeye" | "top" | "aerial" => Some(Self::BirdEye),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AngleApprovalStatus {
    #[default]
    Pending,
    Submitted,
    Approved,
    RegenerateRequested,
}

impl AngleApprovalStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Submitted => "Submitted for Review",
            Self::Approved => "Approved",
            Self::RegenerateRequested => "Regeneration Requested",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiViewSlot {
    pub angle: AngleKind,
    pub image_path: Option<String>,
    pub notes: String,
    pub status: AngleApprovalStatus,
    pub revision: u32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiViewSet {
    pub slots: Vec<MultiViewSlot>,
    pub locked: bool,
}

impl Default for MultiViewSet {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            slots: AngleKind::all()
                .iter()
                .map(|&angle| MultiViewSlot {
                    angle,
                    image_path: None,
                    notes: String::new(),
                    status: AngleApprovalStatus::Pending,
                    revision: 0,
                    updated_at: now.clone(),
                })
                .collect(),
            locked: false,
        }
    }
}

impl MultiViewSet {
    pub fn all_approved(&self) -> bool {
        !self.slots.is_empty()
            && self
                .slots
                .iter()
                .all(|s| s.status == AngleApprovalStatus::Approved)
    }

    pub fn slot(&self, angle: AngleKind) -> Option<&MultiViewSlot> {
        self.slots.iter().find(|s| s.angle == angle)
    }

    pub fn slot_mut(&mut self, angle: AngleKind) -> Option<&mut MultiViewSlot> {
        self.slots.iter_mut().find(|s| s.angle == angle)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StageError {
    #[error(
        "Stage gate blocked: {target:?} requires {required:?} to be Approved & Locked ({reason})"
    )]
    UpstreamStageNotApproved {
        target: PipelineStage,
        required: PipelineStage,
        reason: String,
    },
    #[error("Artifact is locked at {stage:?}; explicit user unlock is required before mutation")]
    ArtifactLocked { stage: PipelineStage },
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LutetianProject {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub current_stage: PipelineStage,
    pub stage_gates: BTreeMap<PipelineStage, StageGateStatus>,
    pub bible: ProjectBible,
    pub multi_view: MultiViewSet,
    pub blueprint: MasterBlueprint,
    pub assets: Vec<AssetSpec>,
    pub telemetry: AgentTelemetryState,
    pub last_validation: ValidationReport,
}

impl Default for LutetianProject {
    fn default() -> Self {
        Self::new_empty("Untitled Environment")
    }
}

impl LutetianProject {
    pub fn new_empty(name: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        let name_str = name.into();
        let mut stage_gates = BTreeMap::new();
        for &stage in PipelineStage::all() {
            stage_gates.insert(
                stage,
                if stage == PipelineStage::Stage1Ingestion {
                    StageGateStatus::InProgress
                } else {
                    StageGateStatus::NotStarted
                },
            );
        }

        let mut project = Self {
            id: Uuid::new_v4().to_string(),
            name: name_str.clone(),
            created_at: now.clone(),
            updated_at: now,
            current_stage: PipelineStage::Stage1Ingestion,
            stage_gates,
            bible: ProjectBible {
                title: name_str,
                ..Default::default()
            },
            multi_view: MultiViewSet::default(),
            blueprint: MasterBlueprint::default(),
            assets: Vec::new(),
            telemetry: AgentTelemetryState::default(),
            last_validation: ValidationReport::default(),
        };
        project.recompute_validation();
        project
    }

    pub fn stage_status(&self, stage: PipelineStage) -> StageGateStatus {
        self.stage_gates
            .get(&stage)
            .copied()
            .unwrap_or(StageGateStatus::NotStarted)
    }

    pub fn is_stage_locked(&self, stage: PipelineStage) -> bool {
        self.stage_status(stage) == StageGateStatus::ApprovedLocked
    }

    pub fn recompute_validation(&mut self) -> &ValidationReport {
        self.last_validation = validate_project(self);
        &self.last_validation
    }

    pub fn submit_project_bible(
        &mut self,
        art_style: String,
        mood: String,
        lighting: String,
        palette_notes: String,
        reference_images: Vec<String>,
        scale_anchors: ScaleAnchors,
    ) -> Result<(), StageError> {
        if self.bible.locked || self.is_stage_locked(PipelineStage::Stage1Ingestion) {
            return Err(StageError::ArtifactLocked {
                stage: PipelineStage::Stage1Ingestion,
            });
        }
        if art_style.trim().is_empty() {
            return Err(StageError::InvalidInput(
                "art_style must not be empty".to_string(),
            ));
        }
        if scale_anchors.human_height_m <= 0.0 || scale_anchors.door_height_m <= 0.0 {
            return Err(StageError::InvalidInput(
                "scale anchors must be positive metric values".to_string(),
            ));
        }

        self.bible.art_style = art_style;
        self.bible.mood = mood;
        self.bible.lighting = lighting;
        self.bible.palette_notes = palette_notes;
        self.bible.reference_images = reference_images;
        self.bible.scale_anchors = scale_anchors;
        self.bible.updated_at = Utc::now().to_rfc3339();
        self.stage_gates.insert(
            PipelineStage::Stage1Ingestion,
            StageGateStatus::AwaitingApproval,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn approve_and_lock_stage1(&mut self) -> Result<(), StageError> {
        if self.bible.art_style.trim().is_empty() && self.bible.reference_images.is_empty() {
            return Err(StageError::InvalidInput(
                "Stage 1 requires art_style or at least one reference image before locking"
                    .to_string(),
            ));
        }
        self.bible.locked = true;
        self.bible.updated_at = Utc::now().to_rfc3339();
        self.stage_gates.insert(
            PipelineStage::Stage1Ingestion,
            StageGateStatus::ApprovedLocked,
        );
        if self.current_stage < PipelineStage::Stage2MultiView {
            self.current_stage = PipelineStage::Stage2MultiView;
            self.stage_gates
                .insert(PipelineStage::Stage2MultiView, StageGateStatus::InProgress);
        }
        self.telemetry.push_log(
            "User",
            "stage-gate",
            "approve_stage1",
            "Stage 1 Project Bible & Scale Anchors approved and locked.",
            ActivityLevel::Success,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn unlock_stage_by_user(&mut self, stage: PipelineStage) {
        match stage {
            PipelineStage::Stage1Ingestion => {
                self.bible.locked = false;
                self.stage_gates
                    .insert(stage, StageGateStatus::AwaitingApproval);
            }
            PipelineStage::Stage2MultiView => {
                self.multi_view.locked = false;
                self.stage_gates
                    .insert(stage, StageGateStatus::AwaitingApproval);
            }
            _ => {
                self.stage_gates.insert(stage, StageGateStatus::InProgress);
            }
        }
        self.telemetry.push_log(
            "User",
            "stage-gate",
            "unlock_stage",
            format!("User unlocked {} for editing.", stage.title()),
            ActivityLevel::Warning,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
    }

    pub fn submit_angle_image(
        &mut self,
        angle: AngleKind,
        image_path: String,
        notes: String,
    ) -> Result<(), StageError> {
        if !self.is_stage_locked(PipelineStage::Stage1Ingestion) {
            return Err(StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage2MultiView,
                required: PipelineStage::Stage1Ingestion,
                reason:
                    "Lock Stage 1 Project Bible & Scale Anchors before submitting multi-view angles"
                        .to_string(),
            });
        }
        if self.multi_view.locked || self.is_stage_locked(PipelineStage::Stage2MultiView) {
            return Err(StageError::ArtifactLocked {
                stage: PipelineStage::Stage2MultiView,
            });
        }
        if image_path.trim().is_empty() {
            return Err(StageError::InvalidInput(
                "image_path must not be empty".to_string(),
            ));
        }

        if let Some(slot) = self.multi_view.slot_mut(angle) {
            slot.image_path = Some(image_path);
            slot.notes = notes;
            slot.status = AngleApprovalStatus::Submitted;
            slot.revision += 1;
            slot.updated_at = Utc::now().to_rfc3339();
        }
        self.stage_gates.insert(
            PipelineStage::Stage2MultiView,
            StageGateStatus::AwaitingApproval,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn approve_angle(&mut self, angle: AngleKind) -> Result<(), StageError> {
        if !self.is_stage_locked(PipelineStage::Stage1Ingestion) {
            return Err(StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage2MultiView,
                required: PipelineStage::Stage1Ingestion,
                reason: "Stage 1 must be locked first".to_string(),
            });
        }
        if let Some(slot) = self.multi_view.slot_mut(angle) {
            slot.status = AngleApprovalStatus::Approved;
            slot.updated_at = Utc::now().to_rfc3339();
        }
        if self.multi_view.all_approved() {
            self.multi_view.locked = true;
            self.stage_gates.insert(
                PipelineStage::Stage2MultiView,
                StageGateStatus::ApprovedLocked,
            );
            if self.current_stage < PipelineStage::Stage3Blueprint {
                self.current_stage = PipelineStage::Stage3Blueprint;
                self.stage_gates
                    .insert(PipelineStage::Stage3Blueprint, StageGateStatus::InProgress);
            }
            self.telemetry.push_log(
                "User",
                "stage-gate",
                "approve_stage2",
                "All 5 multi-view angles approved; Stage 2 locked & Stage 3 unlocked.",
                ActivityLevel::Success,
            );
        }
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn request_angle_regeneration(&mut self, angle: AngleKind, reason: String) {
        self.multi_view.locked = false;
        if let Some(slot) = self.multi_view.slot_mut(angle) {
            slot.status = AngleApprovalStatus::RegenerateRequested;
            if !reason.trim().is_empty() {
                slot.notes = reason.clone();
            }
            slot.updated_at = Utc::now().to_rfc3339();
        }
        self.stage_gates
            .insert(PipelineStage::Stage2MultiView, StageGateStatus::InProgress);
        self.telemetry.push_log(
            "User",
            "lutetian-stage2-multiview",
            "request_regeneration",
            format!("Requested regeneration for {}: {}", angle.label(), reason),
            ActivityLevel::Warning,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
    }

    pub fn submit_blueprint(&mut self, blueprint: MasterBlueprint) -> Result<(), StageError> {
        if !self.is_stage_locked(PipelineStage::Stage2MultiView) {
            return Err(StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage3Blueprint,
                required: PipelineStage::Stage2MultiView,
                reason: "All multi-view angles must be approved & locked before submitting Master Blueprint".to_string(),
            });
        }
        if self.is_stage_locked(PipelineStage::Stage3Blueprint) {
            return Err(StageError::ArtifactLocked {
                stage: PipelineStage::Stage3Blueprint,
            });
        }
        if blueprint.world_width_m <= 0.0 || blueprint.world_depth_m <= 0.0 {
            return Err(StageError::InvalidInput(
                "Blueprint world dimensions must be positive meters".to_string(),
            ));
        }

        self.blueprint = blueprint;
        self.stage_gates.insert(
            PipelineStage::Stage3Blueprint,
            StageGateStatus::AwaitingApproval,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn approve_and_lock_stage3(&mut self) -> Result<(), StageError> {
        if !self.is_stage_locked(PipelineStage::Stage2MultiView) {
            return Err(StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage3Blueprint,
                required: PipelineStage::Stage2MultiView,
                reason: "Stage 2 must be locked first".to_string(),
            });
        }
        if self.blueprint.footprints.is_empty() {
            return Err(StageError::InvalidInput(
                "Master Blueprint needs at least one footprint before locking".to_string(),
            ));
        }
        self.stage_gates.insert(
            PipelineStage::Stage3Blueprint,
            StageGateStatus::ApprovedLocked,
        );
        if self.current_stage < PipelineStage::Stage4Assets {
            self.current_stage = PipelineStage::Stage4Assets;
            self.stage_gates
                .insert(PipelineStage::Stage4Assets, StageGateStatus::InProgress);
        }
        self.telemetry.push_log(
            "User",
            "stage-gate",
            "approve_stage3",
            "Master Blueprint approved & locked; Stage 4 Asset Breakdown unlocked.",
            ActivityLevel::Success,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn upsert_asset_spec(&mut self, asset: AssetSpec) -> Result<(), StageError> {
        if !self.is_stage_locked(PipelineStage::Stage3Blueprint) {
            return Err(StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage4Assets,
                required: PipelineStage::Stage3Blueprint,
                reason: "Stage 3 Master Blueprint must be locked before registering Asset Specs"
                    .to_string(),
            });
        }
        if self.is_stage_locked(PipelineStage::Stage4Assets) {
            return Err(StageError::ArtifactLocked {
                stage: PipelineStage::Stage4Assets,
            });
        }
        if asset.asset_id.trim().is_empty() || !asset.bbox_m.is_positive() {
            return Err(StageError::InvalidInput(
                "Asset requires non-empty asset_id and strictly positive metric bounding box"
                    .to_string(),
            ));
        }

        if let Some(existing) = self
            .assets
            .iter_mut()
            .find(|a| a.asset_id == asset.asset_id)
        {
            *existing = asset;
        } else {
            self.assets.push(asset);
        }
        self.stage_gates.insert(
            PipelineStage::Stage4Assets,
            StageGateStatus::AwaitingApproval,
        );
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        Ok(())
    }

    pub fn approve_and_lock_stage4(&mut self) -> Result<(), StageError> {
        if !self.is_stage_locked(PipelineStage::Stage3Blueprint) {
            return Err(StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage4Assets,
                required: PipelineStage::Stage3Blueprint,
                reason: "Stage 3 must be locked first".to_string(),
            });
        }
        if self.assets.is_empty() {
            return Err(StageError::InvalidInput(
                "Register at least one unique AssetSpec before locking Stage 4".to_string(),
            ));
        }
        self.stage_gates
            .insert(PipelineStage::Stage4Assets, StageGateStatus::ApprovedLocked);
        self.current_stage = PipelineStage::Stage5Validation;
        self.recompute_validation();
        if self.last_validation.passed {
            self.stage_gates.insert(
                PipelineStage::Stage5Validation,
                StageGateStatus::ApprovedLocked,
            );
            self.current_stage = PipelineStage::Stage6EngineSync;
            self.stage_gates
                .insert(PipelineStage::Stage6EngineSync, StageGateStatus::InProgress);
        } else {
            self.stage_gates
                .insert(PipelineStage::Stage5Validation, StageGateStatus::InProgress);
        }
        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn add_footprint_interactive(&mut self) -> String {
        let idx = self.blueprint.footprints.len() + 1;
        let id = format!("NEW_BLDG_{idx:02}");
        self.blueprint.footprints.push(FootprintEntity {
            id: id.clone(),
            asset_id: "BLDG_TIMBER_HOUSE".to_string(),
            label: format!("Modular Structure {idx}"),
            kind: FootprintKind::Building,
            rect: MetricRect2D::new(8.0, 14.0, 6.0, 5.0),
            height_m: 6.4,
            base_elevation_m: 0.15,
            floor_count: 2,
            collidable: true,
        });
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        id
    }

    pub fn add_door_interactive(&mut self, parent_entity_id: Option<&str>) -> String {
        let idx = self.blueprint.doors.len() + 1;
        let id = format!("NEW_DOOR_{idx:02}");
        let (parent, pos, sill) = if let Some(pid) = parent_entity_id
            && let Some(fp) = self.blueprint.footprints.iter().find(|f| f.id == pid)
        {
            (
                fp.id.clone(),
                MetricPoint2D {
                    x_m: fp.rect.x_m + fp.rect.width_m * 0.5,
                    y_m: fp.rect.y_m + fp.rect.depth_m,
                },
                fp.base_elevation_m,
            )
        } else {
            (
                "WORLD".to_string(),
                MetricPoint2D {
                    x_m: 12.0,
                    y_m: 12.0,
                },
                0.0,
            )
        };
        self.blueprint.doors.push(DoorOpening {
            id: id.clone(),
            parent_entity_id: parent,
            label: format!("Doorway {idx}"),
            position_m: pos,
            width_m: self.bible.scale_anchors.door_width_m,
            height_m: self.bible.scale_anchors.door_height_m,
            sill_elevation_m: sill,
        });
        self.updated_at = Utc::now().to_rfc3339();
        self.recompute_validation();
        id
    }

    pub fn delete_entity(&mut self, entity_id: &str) -> bool {
        let fp_before = self.blueprint.footprints.len();
        self.blueprint.footprints.retain(|f| f.id != entity_id);
        let door_before = self.blueprint.doors.len();
        self.blueprint
            .doors
            .retain(|d| d.id != entity_id && d.parent_entity_id != entity_id);
        let zone_before = self.blueprint.zones.len();
        self.blueprint.zones.retain(|z| z.id != entity_id);
        let changed = self.blueprint.footprints.len() != fp_before
            || self.blueprint.doors.len() != door_before
            || self.blueprint.zones.len() != zone_before;
        if changed {
            self.updated_at = Utc::now().to_rfc3339();
            self.recompute_validation();
        }
        changed
    }

    pub fn sync_asset_bbox_from_footprint(&mut self, asset_id: &str) -> bool {
        let Some(fp) = self
            .blueprint
            .footprints
            .iter()
            .find(|f| f.asset_id == asset_id)
            .cloned()
        else {
            return false;
        };
        if let Some(asset) = self.assets.iter_mut().find(|a| a.asset_id == asset_id) {
            asset.bbox_m = MetricDimensions::new(fp.rect.width_m, fp.rect.depth_m, fp.height_m);
            self.updated_at = Utc::now().to_rfc3339();
            self.recompute_validation();
            return true;
        }
        false
    }

    /// Creates a rich, 100% scale-compliant reference demonstration project
    /// ("Lutetian Courtyard Plaza") so the CAD Blueprint viewer, Multi-View gallery,
    /// Asset Catalog BOM, and Live Mission Control are immediately verifiable on first launch.
    pub fn create_demo_project() -> Self {
        let mut p = Self::new_empty("Courtyard Market District (Sample)");
        p.bible = ProjectBible {
            title: "Courtyard Market District".to_string(),
            art_style: "Stylized European Stone & Half-Timbered Realism (PBR)".to_string(),
            mood: "Warm late-afternoon golden sun with crisp architectural shadows".to_string(),
            lighting: "Directional key 38° elevation, warm fill bounce off cobblestone plaza".to_string(),
            palette_notes: "Limestone ashlar #D6CFC2, weathered oak #5C4332, slate roof #3A3D4A, moss foliage #4F6F46".to_string(),
            reference_images: vec!["samples/courtyard_concept_main.png".to_string()],
            scale_anchors: ScaleAnchors::default(),
            locked: true,
            updated_at: Utc::now().to_rfc3339(),
        };
        p.stage_gates.insert(
            PipelineStage::Stage1Ingestion,
            StageGateStatus::ApprovedLocked,
        );

        let now = Utc::now().to_rfc3339();
        p.multi_view = MultiViewSet {
            slots: AngleKind::all()
                .iter()
                .map(|&angle| MultiViewSlot {
                    angle,
                    image_path: Some(format!("samples/courtyard_{}.png", angle.key())),
                    notes: format!("Verified {} proportion & silhouette match.", angle.label()),
                    status: AngleApprovalStatus::Approved,
                    revision: 1,
                    updated_at: now.clone(),
                })
                .collect(),
            locked: true,
        };
        p.stage_gates.insert(
            PipelineStage::Stage2MultiView,
            StageGateStatus::ApprovedLocked,
        );

        p.blueprint = MasterBlueprint {
            grid_unit_m: 1.0,
            world_width_m: 44.0,
            world_depth_m: 32.0,
            origin_label: "SW Corner (0m, 0m)".to_string(),
            zones: vec![
                ZoneRegion {
                    id: "ZONE_GRASS_01".to_string(),
                    label: "North Park Lawn".to_string(),
                    kind: ZoneSurfaceKind::Grass,
                    rect: MetricRect2D::new(0.0, 0.0, 44.0, 12.0),
                    surface_material: "M_Terrain_Grass_Trim".to_string(),
                },
                ZoneRegion {
                    id: "ZONE_PLAZA_01".to_string(),
                    label: "Central Market Plaza".to_string(),
                    kind: ZoneSurfaceKind::Plaza,
                    rect: MetricRect2D::new(0.0, 12.0, 44.0, 12.0),
                    surface_material: "M_Cobble_Plaza".to_string(),
                },
                ZoneRegion {
                    id: "ZONE_ROAD_01".to_string(),
                    label: "Main Carriage Road (4.0m)".to_string(),
                    kind: ZoneSurfaceKind::Road,
                    rect: MetricRect2D::new(0.0, 24.0, 44.0, 4.5),
                    surface_material: "M_Road_Stone_Paving".to_string(),
                },
                ZoneRegion {
                    id: "ZONE_WATER_01".to_string(),
                    label: "Reflecting Canal".to_string(),
                    kind: ZoneSurfaceKind::Water,
                    rect: MetricRect2D::new(0.0, 28.5, 44.0, 3.5),
                    surface_material: "M_Water_Canal".to_string(),
                },
            ],
            elevations: vec![
                ElevationContour {
                    id: "ELEV_NORTH_TERRACE".to_string(),
                    label: "+1.20m North Terrace".to_string(),
                    rect: MetricRect2D::new(2.0, 1.5, 40.0, 9.5),
                    elevation_m: 1.2,
                },
                ElevationContour {
                    id: "ELEV_PLAZA_PAD".to_string(),
                    label: "+0.15m Cobble Pad".to_string(),
                    rect: MetricRect2D::new(4.0, 12.5, 36.0, 11.0),
                    elevation_m: 0.15,
                },
            ],
            footprints: vec![
                FootprintEntity {
                    id: "HOUSE_A_01".to_string(),
                    asset_id: "BLDG_TIMBER_HOUSE".to_string(),
                    label: "Guild Tavern (2 Floors)".to_string(),
                    kind: FootprintKind::Building,
                    rect: MetricRect2D::new(5.0, 3.5, 11.0, 7.5),
                    height_m: 7.4,
                    base_elevation_m: 1.2,
                    floor_count: 2,
                    collidable: true,
                },
                FootprintEntity {
                    id: "HOUSE_B_01".to_string(),
                    asset_id: "BLDG_BAKERY_SHOP".to_string(),
                    label: "Corner Bakery (2 Floors)".to_string(),
                    kind: FootprintKind::Building,
                    rect: MetricRect2D::new(21.0, 3.5, 9.5, 7.0),
                    height_m: 6.8,
                    base_elevation_m: 1.2,
                    floor_count: 2,
                    collidable: true,
                },
                FootprintEntity {
                    id: "TOWER_GATE_01".to_string(),
                    asset_id: "BLDG_WATCHTOWER".to_string(),
                    label: "East Gate Tower".to_string(),
                    kind: FootprintKind::Building,
                    rect: MetricRect2D::new(34.0, 13.0, 6.0, 6.0),
                    height_m: 12.2,
                    base_elevation_m: 0.15,
                    floor_count: 3,
                    collidable: true,
                },
                FootprintEntity {
                    id: "FOUNTAIN_01".to_string(),
                    asset_id: "PROP_STONE_FOUNTAIN".to_string(),
                    label: "Octagonal Fountain".to_string(),
                    kind: FootprintKind::Prop,
                    rect: MetricRect2D::new(19.0, 15.5, 4.0, 4.0),
                    height_m: 2.3,
                    base_elevation_m: 0.15,
                    floor_count: 1,
                    collidable: true,
                },
                FootprintEntity {
                    id: "TREE_01".to_string(),
                    asset_id: "TREE_PINE_01".to_string(),
                    label: "Pine Tree".to_string(),
                    kind: FootprintKind::Foliage,
                    rect: MetricRect2D::new(2.0, 14.0, 2.6, 2.6),
                    height_m: 6.5,
                    base_elevation_m: 0.15,
                    floor_count: 1,
                    collidable: true,
                },
                FootprintEntity {
                    id: "TREE_02".to_string(),
                    asset_id: "TREE_PINE_01".to_string(),
                    label: "Pine Tree".to_string(),
                    kind: FootprintKind::Foliage,
                    rect: MetricRect2D::new(2.0, 20.0, 2.6, 2.6),
                    height_m: 6.5,
                    base_elevation_m: 0.15,
                    floor_count: 1,
                    collidable: true,
                },
                FootprintEntity {
                    id: "TREE_03".to_string(),
                    asset_id: "TREE_PINE_01".to_string(),
                    label: "Pine Tree".to_string(),
                    kind: FootprintKind::Foliage,
                    rect: MetricRect2D::new(32.5, 4.0, 2.6, 2.6),
                    height_m: 6.5,
                    base_elevation_m: 1.2,
                    floor_count: 1,
                    collidable: true,
                },
                FootprintEntity {
                    id: "BENCH_01".to_string(),
                    asset_id: "BENCH_WOOD_01".to_string(),
                    label: "Plaza Bench".to_string(),
                    kind: FootprintKind::Prop,
                    rect: MetricRect2D::new(13.5, 16.5, 2.2, 0.8),
                    height_m: 0.9,
                    base_elevation_m: 0.15,
                    floor_count: 1,
                    collidable: true,
                },
                FootprintEntity {
                    id: "BENCH_02".to_string(),
                    asset_id: "BENCH_WOOD_01".to_string(),
                    label: "Plaza Bench".to_string(),
                    kind: FootprintKind::Prop,
                    rect: MetricRect2D::new(26.0, 16.5, 2.2, 0.8),
                    height_m: 0.9,
                    base_elevation_m: 0.15,
                    floor_count: 1,
                    collidable: true,
                },
            ],
            doors: vec![
                DoorOpening {
                    id: "DOOR_HOUSE_A".to_string(),
                    parent_entity_id: "HOUSE_A_01".to_string(),
                    label: "Tavern Entry (1.3m × 2.25m)".to_string(),
                    position_m: MetricPoint2D {
                        x_m: 10.5,
                        y_m: 11.0,
                    },
                    width_m: 1.3,
                    height_m: 2.25,
                    sill_elevation_m: 1.2,
                },
                DoorOpening {
                    id: "DOOR_BAKERY".to_string(),
                    parent_entity_id: "HOUSE_B_01".to_string(),
                    label: "Bakery Door (1.1m × 2.15m)".to_string(),
                    position_m: MetricPoint2D {
                        x_m: 25.5,
                        y_m: 10.5,
                    },
                    width_m: 1.1,
                    height_m: 2.15,
                    sill_elevation_m: 1.2,
                },
                DoorOpening {
                    id: "DOOR_TOWER".to_string(),
                    parent_entity_id: "TOWER_GATE_01".to_string(),
                    label: "Tower Arch (1.6m × 2.6m)".to_string(),
                    position_m: MetricPoint2D {
                        x_m: 34.0,
                        y_m: 16.0,
                    },
                    width_m: 1.6,
                    height_m: 2.6,
                    sill_elevation_m: 0.15,
                },
            ],
            navigation_paths: vec![
                NavigationPath {
                    id: "NAV_PLAZA_PROMENADE".to_string(),
                    label: "Pedestrian Promenade (2.8m)".to_string(),
                    points_m: vec![
                        MetricPoint2D {
                            x_m: 6.0,
                            y_m: 13.8,
                        },
                        MetricPoint2D {
                            x_m: 17.0,
                            y_m: 13.8,
                        },
                        MetricPoint2D {
                            x_m: 29.0,
                            y_m: 13.8,
                        },
                        MetricPoint2D {
                            x_m: 33.2,
                            y_m: 16.0,
                        },
                    ],
                    width_m: 2.8,
                    is_vehicular: false,
                },
                NavigationPath {
                    id: "NAV_CARRIAGE_LANE".to_string(),
                    label: "Carriage Roadway (4.0m)".to_string(),
                    points_m: vec![
                        MetricPoint2D {
                            x_m: 1.5,
                            y_m: 26.25,
                        },
                        MetricPoint2D {
                            x_m: 42.5,
                            y_m: 26.25,
                        },
                    ],
                    width_m: 4.0,
                    is_vehicular: true,
                },
            ],
            scale_markers: vec![
                ScaleMarker {
                    id: "SCALE_HUMAN_01".to_string(),
                    kind: ScaleMarkerKind::Human,
                    x_m: 10.5,
                    y_m: 13.2,
                    label: "1.8m Human Reference".to_string(),
                },
                ScaleMarker {
                    id: "SCALE_VEHICLE_01".to_string(),
                    kind: ScaleMarkerKind::Vehicle,
                    x_m: 18.0,
                    y_m: 25.3,
                    label: "4.5m Cart / Vehicle".to_string(),
                },
            ],
            placements: vec![AssetPlacement {
                instance_id: "INST_TREE_04".to_string(),
                asset_id: "TREE_PINE_01".to_string(),
                x_m: 39.5,
                y_m: 4.5,
                z_m: 1.2,
                rotation_deg: 25.0,
                scale_xyz: [1.0, 1.0, 1.0],
            }],
        };

        p.assets = vec![
            AssetSpec {
                asset_id: "BLDG_TIMBER_HOUSE".to_string(),
                name: "Guild Tavern Building".to_string(),
                category: AssetCategory::Architecture,
                bbox_m: MetricDimensions::new(11.0, 7.5, 7.4),
                pivot: "bottom_sw".to_string(),
                isolated_image_path: Some("samples/bldg_timber_house.png".to_string()),
                material_tags: vec!["limestone".into(), "oak_timber".into(), "slate_tile".into()],
                notes: "Modular 2-story timber-framed tavern with recessed stone doorway."
                    .to_string(),
            },
            AssetSpec {
                asset_id: "BLDG_BAKERY_SHOP".to_string(),
                name: "Corner Bakery Shop".to_string(),
                category: AssetCategory::Architecture,
                bbox_m: MetricDimensions::new(9.5, 7.0, 6.8),
                pivot: "bottom_sw".to_string(),
                isolated_image_path: Some("samples/bldg_bakery_shop.png".to_string()),
                material_tags: vec!["plaster_warm".into(), "terracotta".into()],
                notes: "2-story corner storefront with awning overhang.".to_string(),
            },
            AssetSpec {
                asset_id: "BLDG_WATCHTOWER".to_string(),
                name: "East Gate Watchtower".to_string(),
                category: AssetCategory::Architecture,
                bbox_m: MetricDimensions::new(6.0, 6.0, 12.2),
                pivot: "bottom_sw".to_string(),
                isolated_image_path: Some("samples/bldg_watchtower.png".to_string()),
                material_tags: vec!["ashlar_stone".into(), "iron_grate".into()],
                notes: "3-tier vertical landmark anchor on east plaza approach.".to_string(),
            },
            AssetSpec {
                asset_id: "PROP_STONE_FOUNTAIN".to_string(),
                name: "Octagonal Plaza Fountain".to_string(),
                category: AssetCategory::StreetFurniture,
                bbox_m: MetricDimensions::new(4.0, 4.0, 2.3),
                pivot: "bottom_center".to_string(),
                isolated_image_path: Some("samples/prop_stone_fountain.png".to_string()),
                material_tags: vec!["carved_limestone".into(), "water".into()],
                notes: "Central plaza focal prop.".to_string(),
            },
            AssetSpec {
                asset_id: "TREE_PINE_01".to_string(),
                name: "Stylized Alpine Pine".to_string(),
                category: AssetCategory::Foliage,
                bbox_m: MetricDimensions::new(2.6, 2.6, 6.5),
                pivot: "bottom_center".to_string(),
                isolated_image_path: Some("samples/tree_pine_01.png".to_string()),
                material_tags: vec!["pine_bark".into(), "needles_alpha".into()],
                notes: "Deduplicated hero tree instanced across courtyard perimeter.".to_string(),
            },
            AssetSpec {
                asset_id: "BENCH_WOOD_01".to_string(),
                name: "Oak Courtyard Bench".to_string(),
                category: AssetCategory::StreetFurniture,
                bbox_m: MetricDimensions::new(2.2, 0.8, 0.9),
                pivot: "bottom_center".to_string(),
                isolated_image_path: Some("samples/bench_wood_01.png".to_string()),
                material_tags: vec!["weathered_oak".into(), "wrought_iron".into()],
                notes: "Seat height 0.45m calibrated to 1.8m human scale.".to_string(),
            },
        ];

        p.stage_gates.insert(
            PipelineStage::Stage3Blueprint,
            StageGateStatus::ApprovedLocked,
        );
        p.stage_gates
            .insert(PipelineStage::Stage4Assets, StageGateStatus::ApprovedLocked);
        p.stage_gates.insert(
            PipelineStage::Stage5Validation,
            StageGateStatus::ApprovedLocked,
        );
        p.stage_gates
            .insert(PipelineStage::Stage6EngineSync, StageGateStatus::InProgress);
        p.current_stage = PipelineStage::Stage6EngineSync;

        p.telemetry.tasks = vec![
            ExecutionTask {
                task_id: "demo-stage1".to_string(),
                stage: PipelineStage::Stage1Ingestion,
                title: "Analyze reference & lock 1.8m scale anchors".to_string(),
                status: TaskState::Completed,
                progress_percent: 100,
                status_message: "Locked human=1.8m, door=2.1m, floor=3.2m, lane=3.5m".to_string(),
                started_at: Some(now.clone()),
                completed_at: Some(now.clone()),
                artifact_refs: vec!["references/courtyard_concept_main.png".to_string()],
            },
            ExecutionTask {
                task_id: "demo-stage2".to_string(),
                stage: PipelineStage::Stage2MultiView,
                title: "Complete & approve 5 orthographic/aerial views".to_string(),
                status: TaskState::Completed,
                progress_percent: 100,
                status_message: "Front, Back, Left, Right, and Bird's-Eye approved".to_string(),
                started_at: Some(now.clone()),
                completed_at: Some(now.clone()),
                artifact_refs: vec!["multiview/courtyard_bird_eye.png".to_string()],
            },
            ExecutionTask {
                task_id: "demo-stage3".to_string(),
                stage: PipelineStage::Stage3Blueprint,
                title: "Draft 44m × 32m metric Master Blueprint JSON".to_string(),
                status: TaskState::Completed,
                progress_percent: 100,
                status_message: "9 footprints, 4 zones, 3 doors, 2 nav lanes verified".to_string(),
                started_at: Some(now.clone()),
                completed_at: Some(now.clone()),
                artifact_refs: vec!["blueprint.json".to_string()],
            },
            ExecutionTask {
                task_id: "demo-stage4".to_string(),
                stage: PipelineStage::Stage4Assets,
                title: "Deduplicate Bill of Materials & lock metric bounding boxes".to_string(),
                status: TaskState::InProgress,
                progress_percent: 85,
                status_message: "Verifying Blender/Engine export manifest parity".to_string(),
                started_at: Some(now),
                completed_at: None,
                artifact_refs: vec!["assets/bom.json".to_string()],
            },
            ExecutionTask {
                task_id: "demo-stage6".to_string(),
                stage: PipelineStage::Stage6EngineSync,
                title: "Export final Engine Sync Bundle (JSON + Scale Certificate)".to_string(),
                status: TaskState::Upcoming,
                progress_percent: 0,
                status_message: "Awaiting final Stage 6 export trigger".to_string(),
                started_at: None,
                completed_at: None,
                artifact_refs: Vec::new(),
            },
        ];

        p.telemetry.push_log(
            "System",
            "lutetian-core",
            "create_demo_project",
            "Loaded sample 'Courtyard Market District' (44m × 32m). Connect Antigravity via MCP or edit interactively.",
            ActivityLevel::Info,
        );
        p.telemetry.record_compliance_update(
            "AGENT-001".to_string(),
            ComplianceStatus::Compliant,
            "Sample baseline loaded with full stage telemetry trace.".to_string(),
        );
        p.telemetry.heartbeat.connected = false;
        p.recompute_validation();
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_gate_prevents_skipping_stage1_and_stage2() {
        let mut proj = LutetianProject::new_empty("Gate Test");

        // Cannot submit angle before Stage 1 is locked
        let err = proj
            .submit_angle_image(AngleKind::Front, "front.png".into(), "notes".into())
            .expect_err("stage 2 must be blocked before stage 1 lock");
        assert!(matches!(
            err,
            StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage2MultiView,
                required: PipelineStage::Stage1Ingestion,
                ..
            }
        ));

        // Submit and lock Stage 1
        proj.submit_project_bible(
            "Stylized PBR".into(),
            "Crisp daylight".into(),
            "Sun 45 deg".into(),
            "Warm limestone".into(),
            vec!["ref.png".into()],
            ScaleAnchors::default(),
        )
        .expect("bible submission");
        proj.approve_and_lock_stage1().expect("lock stage 1");

        // Agent cannot mutate locked Stage 1
        let locked_err = proj
            .submit_project_bible(
                "Changed".into(),
                "Dark".into(),
                "Night".into(),
                "Gray".into(),
                vec![],
                ScaleAnchors::default(),
            )
            .expect_err("locked bible must reject agent mutation");
        assert!(matches!(
            locked_err,
            StageError::ArtifactLocked {
                stage: PipelineStage::Stage1Ingestion
            }
        ));

        // Cannot submit blueprint before all 5 angles are approved
        let bp_err = proj
            .submit_blueprint(MasterBlueprint::default())
            .expect_err("stage 3 must reject before all 5 angles approved");
        assert!(matches!(
            bp_err,
            StageError::UpstreamStageNotApproved {
                target: PipelineStage::Stage3Blueprint,
                required: PipelineStage::Stage2MultiView,
                ..
            }
        ));

        // Submit & approve all 5 angles
        for &angle in AngleKind::all() {
            proj.submit_angle_image(angle, format!("{}.png", angle.key()), "ok".into())
                .expect("submit angle");
            proj.approve_angle(angle).expect("approve angle");
        }
        assert!(proj.is_stage_locked(PipelineStage::Stage2MultiView));
        assert_eq!(proj.current_stage, PipelineStage::Stage3Blueprint);
    }

    #[test]
    fn full_stage_progression_and_engine_bundle_export() {
        let mut proj = LutetianProject::create_demo_project();
        assert_eq!(proj.current_stage, PipelineStage::Stage6EngineSync);

        // User adds & deletes footprint interactively
        let added = proj.add_footprint_interactive();
        assert!(proj.blueprint.footprints.iter().any(|f| f.id == added));
        assert!(proj.delete_entity(&added));
        assert!(!proj.blueprint.footprints.iter().any(|f| f.id == added));

        // Verify Stage 6 Engine Sync bundle writes all 3 files
        let out_dir =
            std::env::temp_dir().join(format!("lutetian_export_{}", uuid::Uuid::new_v4()));
        let manifest_path = crate::export_engine_bundle(&proj, &out_dir).expect("export");
        assert!(manifest_path.exists());
        assert!(out_dir.join("master_blueprint.json").exists());
        assert!(out_dir.join("asset_catalog_bom.json").exists());
        let _ = std::fs::remove_dir_all(&out_dir);
    }
}

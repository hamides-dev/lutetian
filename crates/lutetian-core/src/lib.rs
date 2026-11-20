pub mod assets;
pub mod blueprint;
pub mod config;
pub mod export;
pub mod project;
pub mod storage;
pub mod telemetry;
pub mod updater;
pub mod validator;

pub use assets::{
    AssetCategory, AssetSpec, BomSummaryEntry, MetricDimensions, build_bill_of_materials,
};
pub use blueprint::{
    AssetPlacement, BlueprintLayer, DoorOpening, ElevationContour, FootprintEntity, FootprintKind,
    MasterBlueprint, MetricPoint2D, MetricRect2D, NavigationPath, ScaleMarker, ScaleMarkerKind,
    ZoneRegion, ZoneSurfaceKind,
};
pub use config::{AppConfig, UiThemeMode};
pub use export::{EngineSyncManifest, build_engine_sync_manifest, export_engine_bundle};
pub use project::{
    AngleApprovalStatus, AngleKind, LutetianProject, MultiViewSet, MultiViewSlot, ProjectBible,
    ScaleAnchors, StageError, StageGateStatus,
};
pub use storage::{ActiveBridgeInfo, AppStorage, StorageError};
pub use telemetry::{
    ActivityLevel, ActivityLogEntry, AgentComplianceReport, AgentHeartbeat, AgentTelemetryState,
    ComplianceStatus, ExecutionTask, PipelineStage, TaskState,
};
pub use updater::{
    DownloadProgress, ReleaseAsset, ReleaseManifest, UpdateCheckResult, UpdateError, UpdaterClient,
    has_internet_connectivity,
};
pub use validator::{
    ComplianceRuleEntry, ScaleViolation, ValidationReport, ViolationSeverity, validate_project,
};

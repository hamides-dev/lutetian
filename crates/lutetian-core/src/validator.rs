use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use crate::blueprint::FootprintKind;
use crate::project::{LutetianProject, StageGateStatus};
use crate::telemetry::{ComplianceStatus, PipelineStage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleViolation {
    pub rule_id: String,
    pub entity_id: String,
    pub expected: String,
    pub actual: String,
    pub severity: ViolationSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplianceRuleEntry {
    pub rule_id: String,
    pub title: String,
    pub category: String,
    pub status: ComplianceStatus,
    pub evidence: String,
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct ValidationReport {
    pub passed: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub violations: Vec<ScaleViolation>,
    pub compliance_matrix: Vec<ComplianceRuleEntry>,
}

/// Pure-Rust deterministic mathematical scale & stage-gate validator.
/// Runs zero AI models; every check is exact metric arithmetic.
pub fn validate_project(project: &LutetianProject) -> ValidationReport {
    let mut violations: Vec<ScaleViolation> = Vec::new();
    let mut matrix: Vec<ComplianceRuleEntry> = Vec::new();
    let anchors = &project.bible.scale_anchors;

    // ------------------------------------------------------------------------
    // GATE-001..004: Architectural Pipeline Stage Gates
    // ------------------------------------------------------------------------
    let gate1_status = if project.is_stage_locked(PipelineStage::Stage1Ingestion) {
        ComplianceStatus::Compliant
    } else {
        ComplianceStatus::Pending
    };
    matrix.push(ComplianceRuleEntry {
        rule_id: "GATE-001".to_string(),
        title: "Stage 1 · Project Bible & Scale Anchors Locked".to_string(),
        category: "Stage Gate".to_string(),
        status: gate1_status,
        evidence: format!(
            "human={:.2}m, door={:.2}m, floor={:.2}m, lane={:.2}m ({})",
            anchors.human_height_m,
            anchors.door_height_m,
            anchors.floor_height_m,
            anchors.road_lane_width_m,
            project.stage_status(PipelineStage::Stage1Ingestion).label()
        ),
        entity_id: None,
    });

    let approved_angles = project
        .multi_view
        .slots
        .iter()
        .filter(|s| s.status == crate::project::AngleApprovalStatus::Approved)
        .count();
    let gate2_status = if project.is_stage_locked(PipelineStage::Stage2MultiView) {
        ComplianceStatus::Compliant
    } else {
        ComplianceStatus::Pending
    };
    matrix.push(ComplianceRuleEntry {
        rule_id: "GATE-002".to_string(),
        title: "Stage 2 · 5-Angle Multi-View Approved".to_string(),
        category: "Stage Gate".to_string(),
        status: gate2_status,
        evidence: format!("{approved_angles}/5 orthographic/aerial views approved"),
        entity_id: None,
    });

    let gate3_status = if project.is_stage_locked(PipelineStage::Stage3Blueprint) {
        ComplianceStatus::Compliant
    } else {
        ComplianceStatus::Pending
    };
    matrix.push(ComplianceRuleEntry {
        rule_id: "GATE-003".to_string(),
        title: "Stage 3 · Metric Master Blueprint Submitted".to_string(),
        category: "Stage Gate".to_string(),
        status: gate3_status,
        evidence: format!(
            "{} footprints, {} doors, {} zones on {:.1}m × {:.1}m grid",
            project.blueprint.footprints.len(),
            project.blueprint.doors.len(),
            project.blueprint.zones.len(),
            project.blueprint.world_width_m,
            project.blueprint.world_depth_m
        ),
        entity_id: None,
    });

    let missing_isolated: Vec<&str> = project
        .assets
        .iter()
        .filter(|a| {
            a.isolated_image_path
                .as_deref()
                .map(str::trim)
                .is_none_or(str::is_empty)
        })
        .map(|a| a.asset_id.as_str())
        .collect();
    let gate4_status = if project.assets.is_empty() {
        ComplianceStatus::Pending
    } else if !missing_isolated.is_empty() {
        for asset_id in &missing_isolated {
            violations.push(ScaleViolation {
                rule_id: "GATE-004".to_string(),
                entity_id: (*asset_id).to_string(),
                expected: "Non-empty isolated_image_path slot".to_string(),
                actual: "None".to_string(),
                severity: ViolationSeverity::Warning,
                message: format!(
                    "Asset {asset_id} is missing an isolated reference image slot path."
                ),
            });
        }
        ComplianceStatus::Pending
    } else if project.stage_status(PipelineStage::Stage4Assets) == StageGateStatus::ApprovedLocked {
        ComplianceStatus::Compliant
    } else {
        ComplianceStatus::Pending
    };
    matrix.push(ComplianceRuleEntry {
        rule_id: "GATE-004".to_string(),
        title: "Stage 4 · Deduplicated Assets & Isolated Image Slots".to_string(),
        category: "Stage Gate".to_string(),
        status: gate4_status,
        evidence: if project.assets.is_empty() {
            "No unique AssetSpecs registered yet".to_string()
        } else if missing_isolated.is_empty() {
            format!(
                "{} unique assets registered with bounding boxes and isolated slots",
                project.assets.len()
            )
        } else {
            format!(
                "{} asset(s) missing isolated_image_path: {}",
                missing_isolated.len(),
                missing_isolated.join(", ")
            )
        },
        entity_id: missing_isolated.first().map(|s| (*s).to_string()),
    });

    // ------------------------------------------------------------------------
    // SCALE-001: Scale Anchor Sanity
    // ------------------------------------------------------------------------
    let mut anchor_errs = Vec::new();
    if !(1.2..=2.3).contains(&anchors.human_height_m) {
        anchor_errs.push(ScaleViolation {
            rule_id: "SCALE-001".to_string(),
            entity_id: "ANCHOR_HUMAN".to_string(),
            expected: "1.20m ..= 2.30m".to_string(),
            actual: format!("{:.2}m", anchors.human_height_m),
            severity: ViolationSeverity::Error,
            message: format!(
                "Human anchor height {:.2}m is outside realistic humanoid range [1.20m, 2.30m].",
                anchors.human_height_m
            ),
        });
    }
    if anchors.door_height_m <= anchors.human_height_m {
        anchor_errs.push(ScaleViolation {
            rule_id: "SCALE-001".to_string(),
            entity_id: "ANCHOR_DOOR".to_string(),
            expected: format!("> {:.2}m (human_height)", anchors.human_height_m),
            actual: format!("{:.2}m", anchors.door_height_m),
            severity: ViolationSeverity::Error,
            message: format!(
                "Default door anchor ({:.2}m) must exceed human height ({:.2}m).",
                anchors.door_height_m, anchors.human_height_m
            ),
        });
    }
    if anchors.floor_height_m < anchors.human_height_m * 1.4 {
        anchor_errs.push(ScaleViolation {
            rule_id: "SCALE-001".to_string(),
            entity_id: "ANCHOR_FLOOR".to_string(),
            expected: format!(">= {:.2}m (1.4 × human)", anchors.human_height_m * 1.4),
            actual: format!("{:.2}m", anchors.floor_height_m),
            severity: ViolationSeverity::Error,
            message: format!(
                "Floor-to-floor height ({:.2}m) is below minimum clearance 1.4 × human_height ({:.2}m).",
                anchors.floor_height_m,
                anchors.human_height_m * 1.4
            ),
        });
    }
    if anchors.road_lane_width_m < 2.8 {
        anchor_errs.push(ScaleViolation {
            rule_id: "SCALE-001".to_string(),
            entity_id: "ANCHOR_ROAD".to_string(),
            expected: ">= 2.80m".to_string(),
            actual: format!("{:.2}m", anchors.road_lane_width_m),
            severity: ViolationSeverity::Error,
            message: format!(
                "Road lane width ({:.2}m) is too narrow for vehicle traversal (< 2.80m).",
                anchors.road_lane_width_m
            ),
        });
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-001".to_string(),
        title: "Scale Anchors Baseline Consistency".to_string(),
        category: "Scale Math".to_string(),
        status: if anchor_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: anchor_errs
            .first()
            .map(|v| v.message.clone())
            .unwrap_or_else(|| {
                format!(
                    "Human {:.2}m < Door {:.2}m < Floor {:.2}m; Lane {:.2}m",
                    anchors.human_height_m,
                    anchors.door_height_m,
                    anchors.floor_height_m,
                    anchors.road_lane_width_m
                )
            }),
        entity_id: anchor_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(anchor_errs);

    // ------------------------------------------------------------------------
    // SCALE-002: Building Height >= Human Height * 1.8 (Human cannot be larger than a house!)
    // ------------------------------------------------------------------------
    let min_bldg_h = anchors.human_height_m * 1.8;
    let buildings: Vec<_> = project
        .blueprint
        .footprints
        .iter()
        .filter(|fp| fp.kind == FootprintKind::Building)
        .collect();
    let mut bldg_errs = Vec::new();
    for b in &buildings {
        if b.height_m < min_bldg_h {
            bldg_errs.push(ScaleViolation {
                rule_id: "SCALE-002".to_string(),
                entity_id: b.id.clone(),
                expected: format!(">= {:.2}m (1.8 × {:.2}m human)", min_bldg_h, anchors.human_height_m),
                actual: format!("{:.2}m", b.height_m),
                severity: ViolationSeverity::Error,
                message: format!(
                    "Building '{}' ({}) height {:.2}m is invalid relative to {:.2}m human (min {:.2}m).",
                    b.label, b.id, b.height_m, anchors.human_height_m, min_bldg_h
                ),
            });
        }
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-002".to_string(),
        title: "Building vs. Human Height Ratio (>= 1.8×)".to_string(),
        category: "Scale Math".to_string(),
        status: if buildings.is_empty() {
            ComplianceStatus::Pending
        } else if bldg_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: if buildings.is_empty() {
            "No buildings placed in blueprint yet".to_string()
        } else if let Some(first) = bldg_errs.first() {
            first.message.clone()
        } else {
            format!(
                "All {} building(s) exceed {:.2}m minimum height",
                buildings.len(),
                min_bldg_h
            )
        },
        entity_id: bldg_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(bldg_errs);

    // ------------------------------------------------------------------------
    // SCALE-003: Multi-Floor Height Consistency
    // ------------------------------------------------------------------------
    let mut floor_errs = Vec::new();
    for b in &buildings {
        let floors = b.floor_count.max(1) as f64;
        let min_floor_total = floors * anchors.floor_height_m * 0.85;
        if b.height_m < min_floor_total {
            floor_errs.push(ScaleViolation {
                rule_id: "SCALE-003".to_string(),
                entity_id: b.id.clone(),
                expected: format!(">= {:.2}m for {} floor(s)", min_floor_total, b.floor_count),
                actual: format!("{:.2}m", b.height_m),
                severity: ViolationSeverity::Error,
                message: format!(
                    "Building {} declares {} floor(s) requiring >= {:.2}m, but height is {:.2}m.",
                    b.id, b.floor_count, min_floor_total, b.height_m
                ),
            });
        }
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-003".to_string(),
        title: "Building Floor Count vs. Z Height".to_string(),
        category: "Scale Math".to_string(),
        status: if buildings.is_empty() {
            ComplianceStatus::Pending
        } else if floor_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: floor_errs
            .first()
            .map(|v| v.message.clone())
            .unwrap_or_else(|| {
                format!(
                    "All {} building(s) match declared floor counts",
                    buildings.len()
                )
            }),
        entity_id: floor_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(floor_errs);

    // ------------------------------------------------------------------------
    // SCALE-004: Door Clearance > Human Height
    // ------------------------------------------------------------------------
    let min_door_h = anchors.human_height_m + 0.15;
    let mut door_errs = Vec::new();
    for door in &project.blueprint.doors {
        if door.height_m <= anchors.human_height_m || door.height_m < min_door_h {
            door_errs.push(ScaleViolation {
                rule_id: "SCALE-004".to_string(),
                entity_id: door.id.clone(),
                expected: format!(">= {:.2}m (human + 0.15m clearance)", min_door_h),
                actual: format!("{:.2}m", door.height_m),
                severity: ViolationSeverity::Error,
                message: format!(
                    "Door {} height {:.2}m fails human clearance ({:.2}m human + 0.15m).",
                    door.id, door.height_m, anchors.human_height_m
                ),
            });
        }
        if door.width_m < 0.85 {
            door_errs.push(ScaleViolation {
                rule_id: "SCALE-004".to_string(),
                entity_id: door.id.clone(),
                expected: ">= 0.85m clear width".to_string(),
                actual: format!("{:.2}m", door.width_m),
                severity: ViolationSeverity::Error,
                message: format!(
                    "Door {} width {:.2}m is narrower than 0.85m minimum.",
                    door.id, door.width_m
                ),
            });
        }
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-004".to_string(),
        title: "Door Openings Clear Human Height & Width".to_string(),
        category: "Scale Math".to_string(),
        status: if project.blueprint.doors.is_empty() {
            ComplianceStatus::Pending
        } else if door_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: if project.blueprint.doors.is_empty() {
            "No door openings defined in blueprint".to_string()
        } else if let Some(first) = door_errs.first() {
            first.message.clone()
        } else {
            format!(
                "All {} door(s) clear {:.2}m height & 0.85m width",
                project.blueprint.doors.len(),
                min_door_h
            )
        },
        entity_id: door_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(door_errs);

    // ------------------------------------------------------------------------
    // SCALE-005: Collidable Footprint Overlap Detection
    // ------------------------------------------------------------------------
    let collidable: Vec<_> = project
        .blueprint
        .footprints
        .iter()
        .filter(|fp| fp.collidable)
        .collect();
    let mut overlap_errs = Vec::new();
    for i in 0..collidable.len() {
        for j in (i + 1)..collidable.len() {
            let a = collidable[i];
            let b = collidable[j];
            if a.rect.intersects_xy(&b.rect, 0.02) {
                let a_top = a.base_elevation_m + a.height_m;
                let b_top = b.base_elevation_m + b.height_m;
                let z_overlap =
                    (a.base_elevation_m + 0.02) < b_top && (a_top - 0.02) > b.base_elevation_m;
                if z_overlap {
                    overlap_errs.push(ScaleViolation {
                        rule_id: "SCALE-005".to_string(),
                        entity_id: a.id.clone(),
                        expected: format!("No XY/Z overlap with {}", b.id),
                        actual: "Intersecting 3D volumes".to_string(),
                        severity: ViolationSeverity::Error,
                        message: format!(
                            "Collidable footprints {} and {} intersect in world space.",
                            a.id, b.id
                        ),
                    });
                }
            }
        }
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-005".to_string(),
        title: "Zero Footprint Overlap (Non-Interpenetration)".to_string(),
        category: "Geometry Math".to_string(),
        status: if project.blueprint.footprints.is_empty() {
            ComplianceStatus::Pending
        } else if overlap_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: if project.blueprint.footprints.is_empty() {
            "No footprints to check".to_string()
        } else if let Some(first) = overlap_errs.first() {
            first.message.clone()
        } else {
            format!(
                "Checked {} collidable footprint pairs with 0 overlaps",
                collidable.len()
            )
        },
        entity_id: overlap_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(overlap_errs);

    // ------------------------------------------------------------------------
    // SCALE-006: Asset Bounding Box Matches Blueprint Footprints
    // ------------------------------------------------------------------------
    let asset_map: BTreeMap<&str, &crate::assets::AssetSpec> = project
        .assets
        .iter()
        .map(|a| (a.asset_id.as_str(), a))
        .collect();
    let mut bbox_errs = Vec::new();
    const DIM_TOLERANCE_M: f64 = 0.05;

    for fp in &project.blueprint.footprints {
        if let Some(asset) = asset_map.get(fp.asset_id.as_str()) {
            let dw = (fp.rect.width_m - asset.bbox_m.width_m).abs();
            let dd = (fp.rect.depth_m - asset.bbox_m.depth_m).abs();
            let dh = (fp.height_m - asset.bbox_m.height_m).abs();
            if dw > DIM_TOLERANCE_M || dd > DIM_TOLERANCE_M || dh > DIM_TOLERANCE_M {
                bbox_errs.push(ScaleViolation {
                    rule_id: "SCALE-006".to_string(),
                    entity_id: fp.id.clone(),
                    expected: format!(
                        "{:.2}m × {:.2}m × {:.2}m (AssetSpec {})",
                        asset.bbox_m.width_m,
                        asset.bbox_m.depth_m,
                        asset.bbox_m.height_m,
                        asset.asset_id
                    ),
                    actual: format!(
                        "{:.2}m × {:.2}m × {:.2}m",
                        fp.rect.width_m, fp.rect.depth_m, fp.height_m
                    ),
                    severity: ViolationSeverity::Error,
                    message: format!(
                        "Footprint {} ({:.2}×{:.2}×{:.2}m) differs from AssetSpec {} ({:.2}×{:.2}×{:.2}m).",
                        fp.id,
                        fp.rect.width_m,
                        fp.rect.depth_m,
                        fp.height_m,
                        asset.asset_id,
                        asset.bbox_m.width_m,
                        asset.bbox_m.depth_m,
                        asset.bbox_m.height_m
                    ),
                });
            }
        } else if !project.assets.is_empty() {
            bbox_errs.push(ScaleViolation {
                rule_id: "SCALE-006".to_string(),
                entity_id: fp.id.clone(),
                expected: format!("Registered AssetSpec for '{}'", fp.asset_id),
                actual: "Missing from Asset Catalog".to_string(),
                severity: ViolationSeverity::Warning,
                message: format!(
                    "Blueprint footprint {} references asset_id '{}' which is not yet in the Asset Catalog.",
                    fp.id, fp.asset_id
                ),
            });
        }
    }

    for pl in &project.blueprint.placements {
        if !project.assets.is_empty() && !asset_map.contains_key(pl.asset_id.as_str()) {
            bbox_errs.push(ScaleViolation {
                rule_id: "SCALE-006".to_string(),
                entity_id: pl.instance_id.clone(),
                expected: format!("Registered AssetSpec '{}'", pl.asset_id),
                actual: "Missing from Asset Catalog".to_string(),
                severity: ViolationSeverity::Warning,
                message: format!(
                    "Placement {} references unregistered asset_id '{}'.",
                    pl.instance_id, pl.asset_id
                ),
            });
        }
    }

    let bbox_has_errors = bbox_errs
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error);
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-006".to_string(),
        title: "Asset Bounding Box == Blueprint Footprint Parity".to_string(),
        category: "Catalog Parity".to_string(),
        status: if project.assets.is_empty() || project.blueprint.footprints.is_empty() {
            ComplianceStatus::Pending
        } else if !bbox_has_errors && bbox_errs.is_empty() {
            ComplianceStatus::Compliant
        } else if bbox_has_errors {
            ComplianceStatus::Violated
        } else {
            ComplianceStatus::Pending
        },
        evidence: bbox_errs
            .first()
            .map(|v| v.message.clone())
            .unwrap_or_else(|| {
                format!(
                    "All {} blueprint footprints match AssetSpec bounding boxes (±{DIM_TOLERANCE_M}m)",
                    project.blueprint.footprints.len()
                )
            }),
        entity_id: bbox_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(bbox_errs);

    // ------------------------------------------------------------------------
    // SCALE-007: World Bounds Containment
    // ------------------------------------------------------------------------
    let w_max = project.blueprint.world_width_m;
    let d_max = project.blueprint.world_depth_m;
    let mut bounds_errs = Vec::new();
    for fp in &project.blueprint.footprints {
        let (x0, y0, x1, y1) = fp.rect.world_aabb();
        if x0 < -0.01 || y0 < -0.01 || x1 > w_max + 0.01 || y1 > d_max + 0.01 {
            bounds_errs.push(ScaleViolation {
                rule_id: "SCALE-007".to_string(),
                entity_id: fp.id.clone(),
                expected: format!("Inside [0..{w_max:.1}m, 0..{d_max:.1}m]"),
                actual: format!("[{x0:.2}..{x1:.2}m, {y0:.2}..{y1:.2}m]"),
                severity: ViolationSeverity::Error,
                message: format!("Footprint {} extends outside world bounds.", fp.id),
            });
        }
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-007".to_string(),
        title: "All Entities Inside World Grid Bounds".to_string(),
        category: "Geometry Math".to_string(),
        status: if project.blueprint.footprints.is_empty() {
            ComplianceStatus::Pending
        } else if bounds_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: bounds_errs
            .first()
            .map(|v| v.message.clone())
            .unwrap_or_else(|| format!("All entities contained within {w_max:.1}m × {d_max:.1}m")),
        entity_id: bounds_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(bounds_errs);

    // ------------------------------------------------------------------------
    // SCALE-008: Navigation & Road Width Clearance
    // ------------------------------------------------------------------------
    let mut nav_errs = Vec::new();
    for nav in &project.blueprint.navigation_paths {
        let min_w = if nav.is_vehicular {
            anchors.road_lane_width_m
        } else {
            1.2
        };
        if nav.width_m < min_w {
            nav_errs.push(ScaleViolation {
                rule_id: "SCALE-008".to_string(),
                entity_id: nav.id.clone(),
                expected: format!(">= {min_w:.2}m"),
                actual: format!("{:.2}m", nav.width_m),
                severity: ViolationSeverity::Error,
                message: format!(
                    "Navigation path {} width {:.2}m is narrower than minimum {min_w:.2}m.",
                    nav.id, nav.width_m
                ),
            });
        }
    }
    matrix.push(ComplianceRuleEntry {
        rule_id: "SCALE-008".to_string(),
        title: "Navigation & Road Lane Width Clearance".to_string(),
        category: "Scale Math".to_string(),
        status: if project.blueprint.navigation_paths.is_empty() {
            ComplianceStatus::Pending
        } else if nav_errs.is_empty() {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Violated
        },
        evidence: nav_errs
            .first()
            .map(|v| v.message.clone())
            .unwrap_or_else(|| {
                format!(
                    "All {} navigation path(s) satisfy clearance rules",
                    project.blueprint.navigation_paths.len()
                )
            }),
        entity_id: nav_errs.first().map(|v| v.entity_id.clone()),
    });
    violations.extend(nav_errs);

    // ------------------------------------------------------------------------
    // AGENT-001: Live Telemetry & Agent Protocol Compliance
    // ------------------------------------------------------------------------
    let known_ids: HashSet<String> = matrix.iter().map(|m| m.rule_id.clone()).collect();
    if let Some(agent_rule) = project.telemetry.compliance_overrides.get("AGENT-001") {
        matrix.push(ComplianceRuleEntry {
            rule_id: "AGENT-001".to_string(),
            title: "Live MCP Telemetry & Reporting Contract".to_string(),
            category: "Agent Protocol".to_string(),
            status: agent_rule.status,
            evidence: agent_rule.evidence.clone(),
            entity_id: None,
        });
    } else {
        matrix.push(ComplianceRuleEntry {
            rule_id: "AGENT-001".to_string(),
            title: "Live MCP Telemetry & Reporting Contract".to_string(),
            category: "Agent Protocol".to_string(),
            status: if project.telemetry.heartbeat.connected {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::Pending
            },
            evidence: if project.telemetry.heartbeat.connected {
                format!(
                    "Connected: {} ({})",
                    project.telemetry.heartbeat.agent_name,
                    project.telemetry.heartbeat.current_action
                )
            } else {
                "Waiting for Antigravity MCP agent heartbeat".to_string()
            },
            entity_id: None,
        });
    }

    // Include any extra custom rule reports pushed by an agent
    for (rule_id, report) in &project.telemetry.compliance_overrides {
        if rule_id != "AGENT-001" && !known_ids.contains(rule_id) {
            matrix.push(ComplianceRuleEntry {
                rule_id: rule_id.clone(),
                title: format!("Agent Rule · {rule_id}"),
                category: "Agent Check".to_string(),
                status: report.status,
                evidence: report.evidence.clone(),
                entity_id: None,
            });
        }
    }

    let error_count = violations
        .iter()
        .filter(|v| v.severity == ViolationSeverity::Error)
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == ViolationSeverity::Warning)
        .count();

    ValidationReport {
        passed: error_count == 0,
        error_count,
        warning_count,
        violations,
        compliance_matrix: matrix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::{DoorOpening, FootprintEntity, MetricPoint2D, MetricRect2D};
    use crate::project::LutetianProject;

    #[test]
    fn demo_project_passes_all_deterministic_scale_checks() {
        let project = LutetianProject::create_demo_project();
        let report = validate_project(&project);
        assert!(
            report.passed,
            "Demo project should pass cleanly, got violations: {:?}",
            report.violations
        );
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 0);
    }

    #[test]
    fn rejects_human_larger_than_or_close_to_house() {
        let mut project = LutetianProject::create_demo_project();
        // Set house height to 1.5m while human height is 1.8m!
        let house = project
            .blueprint
            .footprints
            .iter_mut()
            .find(|f| f.id == "HOUSE_A_01")
            .expect("house footprint");
        house.height_m = 1.5;

        let report = validate_project(&project);
        assert!(!report.passed);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.rule_id == "SCALE-002" && v.entity_id == "HOUSE_A_01")
        );
    }

    #[test]
    fn rejects_door_shorter_than_human() {
        let mut project = LutetianProject::create_demo_project();
        project.blueprint.doors.push(DoorOpening {
            id: "DOOR_TINY".to_string(),
            parent_entity_id: "HOUSE_A_01".to_string(),
            label: "Tiny Door".to_string(),
            position_m: MetricPoint2D {
                x_m: 8.0,
                y_m: 11.0,
            },
            width_m: 0.70,
            height_m: 1.65,
            sill_elevation_m: 1.2,
        });

        let report = validate_project(&project);
        assert!(!report.passed);
        let door_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "SCALE-004" && v.entity_id == "DOOR_TINY")
            .collect();
        assert_eq!(door_violations.len(), 2); // Both height and width violated
    }

    #[test]
    fn detects_overlapping_collidable_footprints() {
        let mut project = LutetianProject::create_demo_project();
        project.blueprint.footprints.push(FootprintEntity {
            id: "COLLIDE_BOX".to_string(),
            asset_id: "BENCH_WOOD_01".to_string(),
            label: "Overlapping Crate".to_string(),
            kind: FootprintKind::Prop,
            rect: MetricRect2D::new(6.0, 4.0, 2.2, 0.8), // Overlaps HOUSE_A_01 in XY and Z
            height_m: 0.9,
            base_elevation_m: 1.2,
            floor_count: 1,
            collidable: true,
        });

        let report = validate_project(&project);
        assert!(!report.passed);
        assert!(report.violations.iter().any(|v| v.rule_id == "SCALE-005"));
    }

    #[test]
    fn detects_asset_bbox_mismatch_against_blueprint_footprint() {
        let mut project = LutetianProject::create_demo_project();
        if let Some(asset) = project
            .assets
            .iter_mut()
            .find(|a| a.asset_id == "BLDG_TIMBER_HOUSE")
        {
            asset.bbox_m.width_m = 20.0; // Mismatches 11.0m footprint!
        }

        let report = validate_project(&project);
        assert!(!report.passed);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.rule_id == "SCALE-006" && v.entity_id == "HOUSE_A_01")
        );
    }
}

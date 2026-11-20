use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    #[default]
    Stage1Ingestion = 1,
    Stage2MultiView = 2,
    Stage3Blueprint = 3,
    Stage4Assets = 4,
    Stage5Validation = 5,
    Stage6EngineSync = 6,
}

impl PipelineStage {
    pub fn all() -> &'static [PipelineStage] {
        &[
            PipelineStage::Stage1Ingestion,
            PipelineStage::Stage2MultiView,
            PipelineStage::Stage3Blueprint,
            PipelineStage::Stage4Assets,
            PipelineStage::Stage5Validation,
            PipelineStage::Stage6EngineSync,
        ]
    }

    pub fn number(self) -> u8 {
        self as u8
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Stage1Ingestion => "Stage 1 · Project Bible & Scale Anchors",
            Self::Stage2MultiView => "Stage 2 · Multi-View Completion",
            Self::Stage3Blueprint => "Stage 3 · Master Blueprint (2D CAD)",
            Self::Stage4Assets => "Stage 4 · Asset Breakdown & BOM",
            Self::Stage5Validation => "Stage 5 · Deterministic Scale Validation",
            Self::Stage6EngineSync => "Stage 6 · Engine Sync Export",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Stage1Ingestion => "1. Ingestion",
            Self::Stage2MultiView => "2. Multi-View",
            Self::Stage3Blueprint => "3. Blueprint",
            Self::Stage4Assets => "4. Assets BOM",
            Self::Stage5Validation => "5. Validation",
            Self::Stage6EngineSync => "6. Engine Sync",
        }
    }

    pub fn required_skill(self) -> &'static str {
        match self {
            Self::Stage1Ingestion => "lutetian-stage1-analysis",
            Self::Stage2MultiView => "lutetian-stage2-multiview",
            Self::Stage3Blueprint => "lutetian-stage3-blueprint",
            Self::Stage4Assets => "lutetian-stage4-assets",
            Self::Stage5Validation | Self::Stage6EngineSync => "lutetian-live-protocol",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Completed,
    InProgress,
    #[default]
    Upcoming,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionTask {
    pub task_id: String,
    pub stage: PipelineStage,
    pub title: String,
    pub status: TaskState,
    pub progress_percent: u8,
    pub status_message: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityLogEntry {
    pub timestamp: String,
    pub source: String,
    pub skill: String,
    pub tool: String,
    pub message: String,
    pub level: ActivityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentHeartbeat {
    pub connected: bool,
    pub agent_name: String,
    pub active_stage: PipelineStage,
    pub active_skill: String,
    pub active_tool: String,
    pub current_action: String,
    pub progress_percent: u8,
    pub last_seen_at: String,
}

impl Default for AgentHeartbeat {
    fn default() -> Self {
        Self {
            connected: false,
            agent_name: "Antigravity Agent (standby)".to_string(),
            active_stage: PipelineStage::Stage1Ingestion,
            active_skill: "lutetian-live-protocol".to_string(),
            active_tool: "idle".to_string(),
            current_action: "Waiting for MCP connection on localhost bridge".to_string(),
            progress_percent: 0,
            last_seen_at: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceStatus {
    Compliant,
    #[default]
    Pending,
    Violated,
}

impl ComplianceStatus {
    pub fn badge_text(self) -> &'static str {
        match self {
            Self::Compliant => "COMPLIANT",
            Self::Pending => "PENDING",
            Self::Violated => "VIOLATED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentComplianceReport {
    pub rule_id: String,
    pub status: ComplianceStatus,
    pub evidence: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct AgentTelemetryState {
    pub heartbeat: AgentHeartbeat,
    pub tasks: Vec<ExecutionTask>,
    pub activity_log: Vec<ActivityLogEntry>,
    pub compliance_overrides: BTreeMap<String, AgentComplianceReport>,
}

impl AgentTelemetryState {
    pub fn push_log(
        &mut self,
        source: impl Into<String>,
        skill: impl Into<String>,
        tool: impl Into<String>,
        message: impl Into<String>,
        level: ActivityLevel,
    ) {
        self.activity_log.insert(
            0,
            ActivityLogEntry {
                timestamp: Utc::now().to_rfc3339(),
                source: source.into(),
                skill: skill.into(),
                tool: tool.into(),
                message: message.into(),
                level,
            },
        );
        if self.activity_log.len() > 200 {
            self.activity_log.truncate(200);
        }
    }

    pub fn record_task_started(
        &mut self,
        task_id: String,
        stage: PipelineStage,
        title: String,
        plan_queue: Vec<String>,
    ) {
        let now = Utc::now().to_rfc3339();
        self.heartbeat.connected = true;
        self.heartbeat.active_stage = stage;
        self.heartbeat.active_skill = stage.required_skill().to_string();
        self.heartbeat.active_tool = "report_task_started".to_string();
        self.heartbeat.current_action = title.clone();
        self.heartbeat.progress_percent = 5;
        self.heartbeat.last_seen_at = now.clone();

        if let Some(existing) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            existing.stage = stage;
            existing.title = title.clone();
            existing.status = TaskState::InProgress;
            existing.progress_percent = 5;
            existing.status_message = "Task active".to_string();
            existing.started_at = Some(now.clone());
        } else {
            self.tasks.push(ExecutionTask {
                task_id: task_id.clone(),
                stage,
                title: title.clone(),
                status: TaskState::InProgress,
                progress_percent: 5,
                status_message: "Task active".to_string(),
                started_at: Some(now.clone()),
                completed_at: None,
                artifact_refs: Vec::new(),
            });
        }

        // Add any newly queued upcoming items if they are not already tracked
        for (idx, upcoming_title) in plan_queue.into_iter().enumerate() {
            let trimmed = upcoming_title.trim();
            if trimmed.is_empty() {
                continue;
            }
            let already_exists = self.tasks.iter().any(|t| t.title == trimmed);
            if !already_exists {
                self.tasks.push(ExecutionTask {
                    task_id: format!("planned-{}-{idx}", stage.number()),
                    stage,
                    title: trimmed.to_string(),
                    status: TaskState::Upcoming,
                    progress_percent: 0,
                    status_message: "Queued by agent plan".to_string(),
                    started_at: None,
                    completed_at: None,
                    artifact_refs: Vec::new(),
                });
            }
        }

        self.push_log(
            "Agent",
            stage.required_skill(),
            "report_task_started",
            format!("[{task_id}] Started: {title}"),
            ActivityLevel::Info,
        );
    }

    pub fn record_progress(&mut self, task_id: &str, status_message: String, percent: u8) {
        let now = Utc::now().to_rfc3339();
        let clamped = percent.min(100);
        self.heartbeat.connected = true;
        self.heartbeat.active_tool = "report_progress".to_string();
        self.heartbeat.current_action = status_message.clone();
        self.heartbeat.progress_percent = clamped;
        self.heartbeat.last_seen_at = now;

        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            task.status = TaskState::InProgress;
            task.progress_percent = clamped;
            task.status_message = status_message.clone();
        }

        let skill = self.heartbeat.active_skill.clone();
        self.push_log(
            "Agent",
            skill,
            "report_progress",
            format!("[{task_id}] ({clamped}%) {status_message}"),
            ActivityLevel::Info,
        );
    }

    pub fn record_task_completed(
        &mut self,
        task_id: &str,
        summary: String,
        artifact_refs: Vec<String>,
    ) {
        let now = Utc::now().to_rfc3339();
        self.heartbeat.connected = true;
        self.heartbeat.active_tool = "report_task_completed".to_string();
        self.heartbeat.current_action = summary.clone();
        self.heartbeat.progress_percent = 100;
        self.heartbeat.last_seen_at = now.clone();

        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            task.status = TaskState::Completed;
            task.progress_percent = 100;
            task.status_message = summary.clone();
            task.completed_at = Some(now);
            task.artifact_refs = artifact_refs;
        } else {
            self.tasks.push(ExecutionTask {
                task_id: task_id.to_string(),
                stage: self.heartbeat.active_stage,
                title: summary.clone(),
                status: TaskState::Completed,
                progress_percent: 100,
                status_message: summary.clone(),
                started_at: Some(now.clone()),
                completed_at: Some(now),
                artifact_refs,
            });
        }

        let skill = self.heartbeat.active_skill.clone();
        self.push_log(
            "Agent",
            skill,
            "report_task_completed",
            format!("[{task_id}] Completed: {summary}"),
            ActivityLevel::Success,
        );
    }

    pub fn record_compliance_update(
        &mut self,
        rule_id: String,
        status: ComplianceStatus,
        evidence: String,
    ) {
        let now = Utc::now().to_rfc3339();
        self.heartbeat.connected = true;
        self.heartbeat.active_tool = "report_compliance_update".to_string();
        self.heartbeat.last_seen_at = now.clone();

        let level = match status {
            ComplianceStatus::Compliant => ActivityLevel::Success,
            ComplianceStatus::Pending => ActivityLevel::Warning,
            ComplianceStatus::Violated => ActivityLevel::Error,
        };

        self.compliance_overrides.insert(
            rule_id.clone(),
            AgentComplianceReport {
                rule_id: rule_id.clone(),
                status,
                evidence: evidence.clone(),
                updated_at: now,
            },
        );

        let skill = self.heartbeat.active_skill.clone();
        self.push_log(
            "Agent",
            skill,
            "report_compliance_update",
            format!("[{rule_id}] {} — {evidence}", status.badge_text()),
            level,
        );
    }
}

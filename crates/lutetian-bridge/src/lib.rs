use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use lutetian_core::{
    ActivityLevel, AngleKind, AssetSpec, ComplianceStatus, LutetianProject, MasterBlueprint,
    PipelineStage, ScaleAnchors, export_engine_bundle, validate_project,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, broadcast, mpsc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BridgeEvent {
    ProjectMutated {
        reason: String,
        stage: PipelineStage,
    },
    TelemetryUpdated {
        action: String,
    },
    StageDecisionChanged {
        stage: PipelineStage,
        approved: bool,
        notes: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RpcResponse {
    pub fn success(result: Value) -> Self {
        Self {
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(err: impl Into<String>) -> Self {
        Self {
            ok: false,
            result: None,
            error: Some(err.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeState {
    pub project: Arc<RwLock<LutetianProject>>,
    pub events_tx: broadcast::Sender<BridgeEvent>,
    pub ui_notify_tx: mpsc::UnboundedSender<BridgeEvent>,
    pub port: u16,
}

impl BridgeState {
    pub fn new(
        project: Arc<RwLock<LutetianProject>>,
        ui_notify_tx: mpsc::UnboundedSender<BridgeEvent>,
        port: u16,
    ) -> Self {
        let (events_tx, _) = broadcast::channel(128);
        Self {
            project,
            events_tx,
            ui_notify_tx,
            port,
        }
    }

    pub fn emit(&self, ev: BridgeEvent) {
        let _ = self.events_tx.send(ev.clone());
        let _ = self.ui_notify_tx.send(ev);
    }
}

pub struct RunningBridge {
    pub port: u16,
    pub state: BridgeState,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Drop for RunningBridge {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

pub async fn start_bridge_server(
    preferred_port: u16,
    project: Arc<RwLock<LutetianProject>>,
    ui_notify_tx: mpsc::UnboundedSender<BridgeEvent>,
) -> Result<RunningBridge, std::io::Error> {
    let listener = match TcpListener::bind(("127.0.0.1", preferred_port)).await {
        Ok(l) => l,
        Err(_) => TcpListener::bind(("127.0.0.1", 0)).await?,
    };
    let bound_port = listener.local_addr()?.port();
    let state = BridgeState::new(project, ui_notify_tx, bound_port);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/state", get(state_handler))
        .route("/rpc", post(rpc_handler))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    Ok(RunningBridge {
        port: bound_port,
        state,
        shutdown_tx: Some(shutdown_tx),
    })
}

async fn health_handler(State(state): State<BridgeState>) -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "service": "lutetian-bridge",
        "port": state.port,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn state_handler(State(state): State<BridgeState>) -> impl IntoResponse {
    let proj = state.project.read().await;
    Json(proj.clone())
}

async fn rpc_handler(
    State(state): State<BridgeState>,
    Json(req): Json<RpcRequest>,
) -> (StatusCode, Json<RpcResponse>) {
    let resp = dispatch_rpc(&state, req).await;
    let code = if resp.ok {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    (code, Json(resp))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<BridgeState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

async fn handle_ws_connection(mut socket: WebSocket, state: BridgeState) {
    let mut rx = state.events_tx.subscribe();
    let welcome = json!({
        "event": "connected",
        "port": state.port,
        "timestamp": Utc::now().to_rfc3339(),
    });
    if socket
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            Ok(ev) = rx.recv() => {
                if let Ok(text) = serde_json::to_string(&ev)
                    && socket.send(Message::Text(text.into())).await.is_err()
                {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(req) = serde_json::from_str::<RpcRequest>(&text) {
                            let resp = dispatch_rpc(&state, req).await;
                            if let Ok(out) = serde_json::to_string(&resp)
                                && socket.send(Message::Text(out.into())).await.is_err()
                            {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

pub async fn dispatch_rpc(state: &BridgeState, req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        "report_task_started" => {
            let task_id = str_param(&req.params, "task_id").unwrap_or_else(|| "task".to_string());
            let stage = parse_stage_param(&req.params).unwrap_or(PipelineStage::Stage1Ingestion);
            let title = str_param(&req.params, "title").unwrap_or_else(|| "Agent task".to_string());
            let plan_queue = req.params["plan_queue"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            let mut proj = state.project.write().await;
            proj.telemetry
                .record_task_started(task_id.clone(), stage, title.clone(), plan_queue);
            proj.recompute_validation();
            drop(proj);
            state.emit(BridgeEvent::TelemetryUpdated {
                action: format!("Started task {task_id}: {title}"),
            });
            RpcResponse::success(json!({ "recorded": true, "task_id": task_id }))
        }

        "report_progress" => {
            let Some(task_id) = str_param(&req.params, "task_id") else {
                return RpcResponse::failure("Missing required parameter: task_id");
            };
            let status_message =
                str_param(&req.params, "status_message").unwrap_or_else(|| "Working...".into());
            let percent = req.params["percent"].as_u64().unwrap_or(0).min(100) as u8;

            let mut proj = state.project.write().await;
            proj.telemetry
                .record_progress(&task_id, status_message.clone(), percent);
            drop(proj);
            state.emit(BridgeEvent::TelemetryUpdated {
                action: format!("{task_id} ({percent}%): {status_message}"),
            });
            RpcResponse::success(
                json!({ "recorded": true, "task_id": task_id, "percent": percent }),
            )
        }

        "report_task_completed" => {
            let Some(task_id) = str_param(&req.params, "task_id") else {
                return RpcResponse::failure("Missing required parameter: task_id");
            };
            let summary =
                str_param(&req.params, "summary").unwrap_or_else(|| "Task completed".into());
            let artifact_refs = req.params["artifact_refs"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            let mut proj = state.project.write().await;
            proj.telemetry
                .record_task_completed(&task_id, summary.clone(), artifact_refs);
            proj.recompute_validation();
            drop(proj);
            state.emit(BridgeEvent::TelemetryUpdated {
                action: format!("Completed {task_id}: {summary}"),
            });
            RpcResponse::success(json!({ "recorded": true, "task_id": task_id }))
        }

        "report_compliance_update" => {
            let Some(rule_id) = str_param(&req.params, "rule_id") else {
                return RpcResponse::failure("Missing required parameter: rule_id");
            };
            let Some(status_str) = str_param(&req.params, "status") else {
                return RpcResponse::failure("Missing required parameter: status");
            };
            let status = match status_str.trim().to_ascii_uppercase().as_str() {
                "COMPLIANT" | "PASS" | "PASSED" => ComplianceStatus::Compliant,
                "PENDING" | "IN_PROGRESS" => ComplianceStatus::Pending,
                "VIOLATED" | "FAIL" | "FAILED" => ComplianceStatus::Violated,
                other => {
                    return RpcResponse::failure(format!(
                        "Invalid compliance status '{other}'; expected COMPLIANT, PENDING, or VIOLATED"
                    ));
                }
            };
            let evidence = str_param(&req.params, "evidence").unwrap_or_default();

            let mut proj = state.project.write().await;
            proj.telemetry
                .record_compliance_update(rule_id.clone(), status, evidence);
            proj.recompute_validation();
            drop(proj);
            state.emit(BridgeEvent::TelemetryUpdated {
                action: format!("Compliance {rule_id}: {}", status.badge_text()),
            });
            RpcResponse::success(json!({
                "recorded": true,
                "rule_id": rule_id,
                "status": status.badge_text()
            }))
        }

        "get_project_state" => {
            let mut proj = state.project.write().await;
            let was_connected = proj.telemetry.heartbeat.connected;
            proj.telemetry.heartbeat.connected = true;
            proj.telemetry.heartbeat.agent_name = "Antigravity Agent (MCP)".to_string();
            proj.telemetry.heartbeat.active_tool = "get_project_state".to_string();
            proj.telemetry.heartbeat.last_seen_at = Utc::now().to_rfc3339();
            let val = serde_json::to_value(&*proj);
            drop(proj);
            if !was_connected {
                state.emit(BridgeEvent::TelemetryUpdated {
                    action: "Antigravity MCP agent connected".to_string(),
                });
            }
            match val {
                Ok(v) => RpcResponse::success(v),
                Err(e) => RpcResponse::failure(e.to_string()),
            }
        }

        "get_active_stage_skill" => {
            let mut proj = state.project.write().await;
            let was_connected = proj.telemetry.heartbeat.connected;
            proj.telemetry.heartbeat.connected = true;
            proj.telemetry.heartbeat.agent_name = "Antigravity Agent (MCP)".to_string();
            proj.telemetry.heartbeat.active_tool = "get_active_stage_skill".to_string();
            proj.telemetry.heartbeat.last_seen_at = Utc::now().to_rfc3339();
            let stage = proj.current_stage;
            let gate = proj.stage_status(stage);
            let res = json!({
                "current_stage": stage,
                "stage_number": stage.number(),
                "stage_title": stage.title(),
                "gate_status": gate,
                "required_skill": stage.required_skill(),
                "always_required_protocol": "lutetian-live-protocol",
                "scale_anchors": proj.bible.scale_anchors,
                "validation_passed": proj.last_validation.passed,
            });
            drop(proj);
            if !was_connected {
                state.emit(BridgeEvent::TelemetryUpdated {
                    action: "Antigravity MCP agent connected".to_string(),
                });
            }
            RpcResponse::success(res)
        }

        "submit_project_bible" => {
            let art_style = str_param(&req.params, "art_style").unwrap_or_default();
            let mood = str_param(&req.params, "mood").unwrap_or_default();
            let lighting = str_param(&req.params, "lighting").unwrap_or_default();
            let palette_notes = str_param(&req.params, "palette_notes").unwrap_or_default();
            let reference_images = req.params["reference_images"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let scale_anchors = if req.params["scale_anchors"].is_object() {
                serde_json::from_value::<ScaleAnchors>(req.params["scale_anchors"].clone())
                    .unwrap_or_default()
            } else {
                ScaleAnchors::default()
            };

            let mut proj = state.project.write().await;
            match proj.submit_project_bible(
                art_style,
                mood,
                lighting,
                palette_notes,
                reference_images,
                scale_anchors,
            ) {
                Ok(()) => {
                    proj.telemetry.push_log(
                        "Agent",
                        "lutetian-stage1-analysis",
                        "submit_project_bible",
                        "Submitted Project Bible & Metric Scale Anchors for user approval.",
                        ActivityLevel::Success,
                    );
                    let stage = proj.current_stage;
                    drop(proj);
                    state.emit(BridgeEvent::ProjectMutated {
                        reason: "Project Bible updated by agent".to_string(),
                        stage,
                    });
                    RpcResponse::success(json!({
                        "accepted": true,
                        "status": "awaiting_user_approval"
                    }))
                }
                Err(e) => RpcResponse::failure(e.to_string()),
            }
        }

        "submit_angle_image" => {
            let Some(angle_raw) = str_param(&req.params, "angle") else {
                return RpcResponse::failure("Missing required parameter: angle");
            };
            let Some(angle) = AngleKind::from_key(&angle_raw) else {
                return RpcResponse::failure(format!(
                    "Invalid angle '{angle_raw}'; expected front, back, left, right, or bird_eye"
                ));
            };
            let Some(image_path) = str_param(&req.params, "image_path") else {
                return RpcResponse::failure("Missing required parameter: image_path");
            };
            let notes = str_param(&req.params, "notes").unwrap_or_default();

            let mut proj = state.project.write().await;
            match proj.submit_angle_image(angle, image_path.clone(), notes) {
                Ok(()) => {
                    proj.telemetry.push_log(
                        "Agent",
                        "lutetian-stage2-multiview",
                        "submit_angle_image",
                        format!("Submitted {} angle view: {}", angle.label(), image_path),
                        ActivityLevel::Success,
                    );
                    let stage = proj.current_stage;
                    drop(proj);
                    state.emit(BridgeEvent::ProjectMutated {
                        reason: format!("Angle {} submitted", angle.key()),
                        stage,
                    });
                    RpcResponse::success(json!({
                        "accepted": true,
                        "angle": angle.key(),
                        "status": "submitted"
                    }))
                }
                Err(e) => RpcResponse::failure(e.to_string()),
            }
        }

        "submit_blueprint_json" => {
            let bp_value = if req.params.get("blueprint").is_some() {
                req.params["blueprint"].clone()
            } else {
                req.params.clone()
            };
            let blueprint: MasterBlueprint = match serde_json::from_value(bp_value) {
                Ok(b) => b,
                Err(e) => {
                    return RpcResponse::failure(format!("Invalid MasterBlueprint JSON: {e}"));
                }
            };

            let mut proj = state.project.write().await;
            match proj.submit_blueprint(blueprint) {
                Ok(()) => {
                    let report = proj.last_validation.clone();
                    proj.telemetry.push_log(
                        "Agent",
                        "lutetian-stage3-blueprint",
                        "submit_blueprint_json",
                        format!(
                            "Submitted Master Blueprint JSON ({} errors, {} warnings).",
                            report.error_count, report.warning_count
                        ),
                        if report.passed {
                            ActivityLevel::Success
                        } else {
                            ActivityLevel::Warning
                        },
                    );
                    let stage = proj.current_stage;
                    drop(proj);
                    state.emit(BridgeEvent::ProjectMutated {
                        reason: "Master Blueprint JSON updated".to_string(),
                        stage,
                    });
                    RpcResponse::success(json!({
                        "accepted": true,
                        "validation": report
                    }))
                }
                Err(e) => RpcResponse::failure(e.to_string()),
            }
        }

        "submit_asset_spec" => {
            let asset_val = if req.params.get("asset").is_some() {
                req.params["asset"].clone()
            } else {
                req.params.clone()
            };
            let asset: AssetSpec = match serde_json::from_value(asset_val) {
                Ok(a) => a,
                Err(e) => return RpcResponse::failure(format!("Invalid AssetSpec JSON: {e}")),
            };
            let asset_id = asset.asset_id.clone();

            let mut proj = state.project.write().await;
            match proj.upsert_asset_spec(asset) {
                Ok(()) => {
                    let report = proj.last_validation.clone();
                    proj.telemetry.push_log(
                        "Agent",
                        "lutetian-stage4-assets",
                        "submit_asset_spec",
                        format!("Registered AssetSpec {asset_id}."),
                        ActivityLevel::Success,
                    );
                    let stage = proj.current_stage;
                    drop(proj);
                    state.emit(BridgeEvent::ProjectMutated {
                        reason: format!("AssetSpec {asset_id} registered"),
                        stage,
                    });
                    RpcResponse::success(json!({
                        "accepted": true,
                        "asset_id": asset_id,
                        "validation": report
                    }))
                }
                Err(e) => RpcResponse::failure(e.to_string()),
            }
        }

        "run_scale_validation" => {
            let mut proj = state.project.write().await;
            let report = validate_project(&proj);
            proj.last_validation = report.clone();
            proj.telemetry.push_log(
                "Agent",
                "lutetian-core",
                "run_scale_validation",
                format!(
                    "Deterministic validator finished: passed={}, errors={}, warnings={}",
                    report.passed, report.error_count, report.warning_count
                ),
                if report.passed {
                    ActivityLevel::Success
                } else {
                    ActivityLevel::Error
                },
            );
            let stage = proj.current_stage;
            drop(proj);
            state.emit(BridgeEvent::ProjectMutated {
                reason: "Deterministic scale validation executed".to_string(),
                stage,
            });
            match serde_json::to_value(report) {
                Ok(v) => RpcResponse::success(v),
                Err(e) => RpcResponse::failure(e.to_string()),
            }
        }

        "export_engine_bundle" => {
            let out_dir = str_param(&req.params, "output_dir")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("exports"));
            let proj = state.project.read().await;
            match export_engine_bundle(&proj, &out_dir) {
                Ok(manifest_path) => {
                    let path_str = manifest_path.to_string_lossy().to_string();
                    drop(proj);
                    let mut proj_w = state.project.write().await;
                    proj_w.telemetry.push_log(
                        "Agent",
                        "lutetian-core",
                        "export_engine_bundle",
                        format!("Exported Stage 6 Engine Sync bundle to {path_str}"),
                        ActivityLevel::Success,
                    );
                    RpcResponse::success(json!({
                        "exported": true,
                        "manifest_path": path_str
                    }))
                }
                Err(e) => RpcResponse::failure(format!("Export failed: {e}")),
            }
        }

        other => RpcResponse::failure(format!("Unknown RPC method: {other}")),
    }
}

fn str_param(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn parse_stage_param(params: &Value) -> Option<PipelineStage> {
    let val = params.get("stage")?;
    if let Some(n) = val.as_u64() {
        return match n {
            1 => Some(PipelineStage::Stage1Ingestion),
            2 => Some(PipelineStage::Stage2MultiView),
            3 => Some(PipelineStage::Stage3Blueprint),
            4 => Some(PipelineStage::Stage4Assets),
            5 => Some(PipelineStage::Stage5Validation),
            6 => Some(PipelineStage::Stage6EngineSync),
            _ => None,
        };
    }
    if let Some(s) = val.as_str() {
        let lower = s.trim().to_ascii_lowercase();
        if lower.contains('1') || lower.contains("ingestion") || lower.contains("bible") {
            return Some(PipelineStage::Stage1Ingestion);
        }
        if lower.contains('2') || lower.contains("multi") || lower.contains("view") {
            return Some(PipelineStage::Stage2MultiView);
        }
        if lower.contains('3') || lower.contains("blueprint") {
            return Some(PipelineStage::Stage3Blueprint);
        }
        if lower.contains('4') || lower.contains("asset") || lower.contains("bom") {
            return Some(PipelineStage::Stage4Assets);
        }
        if lower.contains('5') || lower.contains("valid") {
            return Some(PipelineStage::Stage5Validation);
        }
        if lower.contains('6') || lower.contains("sync") || lower.contains("export") {
            return Some(PipelineStage::Stage6EngineSync);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;

    #[tokio::test]
    async fn end_to_end_live_telemetry_http_and_websocket() {
        let project = Arc::new(RwLock::new(LutetianProject::create_demo_project()));
        let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();

        let bridge = start_bridge_server(0, Arc::clone(&project), ui_tx)
            .await
            .expect("start bridge");
        let port = bridge.port;

        // Connect WebSocket listener
        let ws_url = format!("ws://127.0.0.1:{port}/ws");
        let (mut ws_stream, _) = connect_async(&ws_url).await.expect("connect ws");
        // Consume welcome message
        let _ = ws_stream.next().await;

        // Send HTTP RPC calls matching Antigravity live reporting contract
        let client = reqwest::Client::new();
        let rpc_url = format!("http://127.0.0.1:{port}/rpc");

        let resp1: RpcResponse = client
            .post(&rpc_url)
            .json(&json!({
                "method": "report_task_started",
                "params": {
                    "task_id": "live-e2e-1",
                    "stage": 3,
                    "title": "Synthesizing metric blueprint",
                    "plan_queue": ["Register tree assets", "Run validator"]
                }
            }))
            .send()
            .await
            .expect("rpc1")
            .json()
            .await
            .expect("json1");
        assert!(resp1.ok);

        let resp2: RpcResponse = client
            .post(&rpc_url)
            .json(&json!({
                "method": "report_progress",
                "params": {
                    "task_id": "live-e2e-1",
                    "status_message": "Checking door clearance",
                    "percent": 75
                }
            }))
            .send()
            .await
            .expect("rpc2")
            .json()
            .await
            .expect("json2");
        assert!(resp2.ok);

        let resp3: RpcResponse = client
            .post(&rpc_url)
            .json(&json!({
                "method": "report_compliance_update",
                "params": {
                    "rule_id": "SCALE-002",
                    "status": "COMPLIANT",
                    "evidence": "Verified 7.4m tavern > 3.24m threshold"
                }
            }))
            .send()
            .await
            .expect("rpc3")
            .json()
            .await
            .expect("json3");
        assert!(resp3.ok);

        // Verify UI notification channel received live events
        let ev1 = tokio::time::timeout(std::time::Duration::from_secs(2), ui_rx.recv())
            .await
            .expect("ui event timeout")
            .expect("ui event");
        assert!(matches!(ev1, BridgeEvent::TelemetryUpdated { .. }));

        // Verify WebSocket client also received broadcast event
        let ws_msg = tokio::time::timeout(std::time::Duration::from_secs(2), ws_stream.next())
            .await
            .expect("ws event timeout")
            .expect("ws msg")
            .expect("ws ok");
        assert!(
            ws_msg
                .to_text()
                .expect("text")
                .contains("telemetry_updated")
        );

        // Verify state mutation
        let guard = project.read().await;
        assert!(guard.telemetry.heartbeat.connected);
        assert_eq!(guard.telemetry.heartbeat.progress_percent, 75);
        assert!(
            guard
                .telemetry
                .tasks
                .iter()
                .any(|t| t.task_id == "live-e2e-1" && t.progress_percent == 75)
        );
    }
}

use lutetian_core::AppStorage;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct BridgeRpcResponse {
    ok: bool,
    result: Option<Value>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let http = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| Client::new());

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut lines = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(err) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {err}"),
                    }),
                };
                if let Ok(raw) = serde_json::to_string(&err_resp) {
                    let _ = stdout.write_all(raw.as_bytes()).await;
                    let _ = stdout.write_all(b"\n").await;
                    let _ = stdout.flush().await;
                }
                continue;
            }
        };

        if let Some(resp) = handle_jsonrpc_request(&http, req).await
            && let Ok(raw) = serde_json::to_string(&resp)
        {
            let _ = stdout.write_all(raw.as_bytes()).await;
            let _ = stdout.write_all(b"\n").await;
            let _ = stdout.flush().await;
        }
    }
}

async fn handle_jsonrpc_request(http: &Client, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = req.id.clone();
    // Notifications without id do not require a JSON-RPC response.
    let id_val = id?;

    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: id_val,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "lutetian-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        }),
        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: id_val,
            result: Some(json!({})),
            error: None,
        }),
        "tools/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: id_val,
            result: Some(json!({
                "tools": mcp_tool_definitions()
            })),
            error: None,
        }),
        "tools/call" => {
            let tool_name = req.params["name"].as_str().unwrap_or("").to_string();
            let args = req
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let tool_result = forward_tool_call(http, &tool_name, args).await;
            Some(JsonRpcResponse {
                jsonrpc: "2.0",
                id: id_val,
                result: Some(tool_result),
                error: None,
            })
        }
        other => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id: id_val,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {other}"),
            }),
        }),
    }
}

fn resolve_bridge_base_url() -> String {
    if let Ok(url) = std::env::var("LUTETIAN_BRIDGE_URL") {
        let trimmed = url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    if let Some(info) = AppStorage::open_default().read_bridge_info() {
        return format!("http://{}:{}", info.host, info.port);
    }
    "http://127.0.0.1:47821".to_string()
}

async fn forward_tool_call(http: &Client, tool_name: &str, args: Value) -> Value {
    let base_url = resolve_bridge_base_url();
    let rpc_url = format!("{base_url}/rpc");

    let resp = match http
        .post(&rpc_url)
        .json(&json!({
            "method": tool_name,
            "params": args
        }))
        .send()
        .await
    {
        Ok(r) => r,
        Err(err) => {
            return json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "Could not connect to Lutetian desktop bridge at {rpc_url}: {err}. Ensure Lutetian desktop app is running."
                    )
                }],
                "isError": true
            });
        }
    };

    match resp.json::<BridgeRpcResponse>().await {
        Ok(parsed) if parsed.ok => {
            let text = serde_json::to_string_pretty(&parsed.result.unwrap_or(Value::Null))
                .unwrap_or_else(|_| "{}".to_string());
            json!({
                "content": [{
                    "type": "text",
                    "text": text
                }],
                "isError": false
            })
        }
        Ok(parsed) => {
            json!({
                "content": [{
                    "type": "text",
                    "text": parsed.error.unwrap_or_else(|| "Bridge RPC rejected request".to_string())
                }],
                "isError": true
            })
        }
        Err(err) => {
            json!({
                "content": [{
                    "type": "text",
                    "text": format!("Invalid JSON response from Lutetian bridge: {err}")
                }],
                "isError": true
            })
        }
    }
}

pub fn mcp_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "report_task_started",
            "description": "MANDATORY: Call before starting any task in Lutetian to publish live status, active stage, and upcoming execution queue to Mission Control.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "string", "description": "Unique identifier for the current task (e.g. stage1-bible)" },
                    "stage": { "type": ["integer", "string"], "description": "Pipeline stage number (1..6) or name" },
                    "title": { "type": "string", "description": "Human-readable description of what the agent is doing now" },
                    "plan_queue": { "type": "array", "items": { "type": "string" }, "description": "Ordered upcoming tasks to populate the Planned column" }
                },
                "required": ["task_id", "stage", "title"]
            }
        }),
        json!({
            "name": "report_progress",
            "description": "MANDATORY: Stream live progress percentage and status message during multi-step work.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "string" },
                    "status_message": { "type": "string" },
                    "percent": { "type": "integer", "minimum": 0, "maximum": 100 }
                },
                "required": ["task_id", "status_message", "percent"]
            }
        }),
        json!({
            "name": "report_task_completed",
            "description": "MANDATORY: Mark a task Completed in Mission Control with summary and generated artifact paths.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "string" },
                    "summary": { "type": "string" },
                    "artifact_refs": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["task_id", "summary"]
            }
        }),
        json!({
            "name": "report_compliance_update",
            "description": "MANDATORY: Update a rule status in Lutetian's Compliance & Rules Matrix.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "rule_id": { "type": "string" },
                    "status": { "type": "string", "enum": ["COMPLIANT", "PENDING", "VIOLATED"] },
                    "evidence": { "type": "string" }
                },
                "required": ["rule_id", "status", "evidence"]
            }
        }),
        json!({
            "name": "get_project_state",
            "description": "Fetch the full LutetianProject JSON state including stage locks, scale anchors, multi-view approvals, blueprint, assets, and validation report.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "get_active_stage_skill",
            "description": "Check the current unlocked pipeline stage, required SKILL.md name, and scale anchor constraints.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "submit_project_bible",
            "description": "Stage 1: Submit extracted art style, mood, lighting, palette notes, reference images, and metric ScaleAnchors for user approval.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "art_style": { "type": "string" },
                    "mood": { "type": "string" },
                    "lighting": { "type": "string" },
                    "palette_notes": { "type": "string" },
                    "reference_images": { "type": "array", "items": { "type": "string" } },
                    "scale_anchors": { "type": "object" }
                },
                "required": ["art_style"]
            }
        }),
        json!({
            "name": "submit_angle_image",
            "description": "Stage 2: Submit a generated or prepared concept view for one of front, back, left, right, or bird_eye.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "angle": { "type": "string", "enum": ["front", "back", "left", "right", "bird_eye"] },
                    "image_path": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["angle", "image_path"]
            }
        }),
        json!({
            "name": "submit_blueprint_json",
            "description": "Stage 3: Submit structured metric MasterBlueprint JSON (requires Stage 2 Approved & Locked).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "blueprint": { "type": "object" }
                },
                "required": ["blueprint"]
            }
        }),
        json!({
            "name": "submit_asset_spec",
            "description": "Stage 4: Register or update a deduplicated AssetSpec with exact metric bounding box (width_m, depth_m, height_m) and isolated_image_path.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset": { "type": "object" }
                },
                "required": ["asset"]
            }
        }),
        json!({
            "name": "run_scale_validation",
            "description": "Stage 5: Run the pure-Rust Deterministic Scale Validator across all blueprints, doors, clearances, overlaps, and asset bounding boxes.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "export_engine_bundle",
            "description": "Stage 6: Export the final Engine Sync JSON manifest, MasterBlueprint JSON, and deduplicated Asset BOM JSON to output_dir.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "output_dir": { "type": "string" }
                }
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_exposes_all_contract_tools() {
        let defs = mcp_tool_definitions();
        let names: Vec<&str> = defs.iter().filter_map(|d| d["name"].as_str()).collect();
        for required in [
            "report_task_started",
            "report_progress",
            "report_task_completed",
            "report_compliance_update",
            "get_project_state",
            "get_active_stage_skill",
            "submit_project_bible",
            "submit_angle_image",
            "submit_blueprint_json",
            "submit_asset_spec",
            "run_scale_validation",
            "export_engine_bundle",
        ] {
            assert!(names.contains(&required), "Missing MCP tool: {required}");
        }
    }

    #[tokio::test]
    async fn handles_initialize_and_tools_list() {
        let client = Client::new();
        let init = handle_jsonrpc_request(
            &client,
            JsonRpcRequest {
                id: Some(json!(1)),
                method: "initialize".to_string(),
                params: json!({}),
            },
        )
        .await
        .expect("init response");
        assert!(init.error.is_none());
        assert_eq!(init.result.unwrap()["serverInfo"]["name"], "lutetian-mcp");

        let list = handle_jsonrpc_request(
            &client,
            JsonRpcRequest {
                id: Some(json!(2)),
                method: "tools/list".to_string(),
                params: json!({}),
            },
        )
        .await
        .expect("list response");
        assert!(list.error.is_none());
        assert_eq!(list.result.unwrap()["tools"].as_array().unwrap().len(), 12);
    }
}

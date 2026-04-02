use clap::Args;
use serde_json::{json, Value};

use crate::api::client::GtmApiClient;
use crate::api::workspace::resolve_workspace;
use crate::error::Result;
use crate::output::formatter::OutputFormat;

#[derive(Args)]
pub struct ExplainArgs {
    /// Tag ID to explain
    #[arg(long)]
    pub tag_id: String,
    #[arg(long, env = "GTM_ACCOUNT_ID")]
    account_id: String,
    #[arg(long, env = "GTM_CONTAINER_ID")]
    container_id: String,
    #[arg(long, env = "GTM_WORKSPACE_ID")]
    workspace_id: Option<String>,
}

pub async fn handle(args: ExplainArgs, client: &GtmApiClient, format: &OutputFormat) -> Result<()> {
    let ws_id = resolve_workspace(
        client,
        &args.account_id,
        &args.container_id,
        args.workspace_id.as_deref(),
    )
    .await?;
    let base = format!(
        "accounts/{}/containers/{}/workspaces/{}",
        args.account_id, args.container_id, ws_id
    );

    // Fetch tag + all triggers + all variables concurrently
    let tag_path = format!("{base}/tags/{}", args.tag_id);
    let triggers_path = format!("{base}/triggers");
    let variables_path = format!("{base}/variables");
    let (tag_res, triggers_res, variables_res) = tokio::join!(
        client.get(&tag_path),
        client.get_all(&triggers_path),
        client.get_all(&variables_path),
    );

    let tag = tag_res?;
    let triggers_data = triggers_res?;
    let variables_data = variables_res?;

    let tag_name = str_field(&tag, "name");
    let tag_type = str_field(&tag, "type");
    let tag_id = str_field(&tag, "tagId");

    // Build trigger index
    let all_triggers: Vec<&Value> = triggers_data
        .get("trigger")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    let trigger_map: std::collections::HashMap<&str, &Value> = all_triggers
        .iter()
        .filter_map(|t| {
            t.get("triggerId")
                .and_then(|v| v.as_str())
                .map(|id| (id, *t))
        })
        .collect();

    // Resolve firing triggers
    let firing_ids = id_array(&tag, "firingTriggerId");
    let firing: Vec<TriggerInfo> = firing_ids
        .iter()
        .map(|id| resolve_trigger(id, &trigger_map))
        .collect();

    // Resolve blocking triggers
    let blocking_ids = id_array(&tag, "blockingTriggerId");
    let blocking: Vec<TriggerInfo> = blocking_ids
        .iter()
        .map(|id| resolve_trigger(id, &trigger_map))
        .collect();

    // Extract referenced variables ({{varName}} patterns)
    let tag_json = tag.to_string();
    let referenced_vars = extract_variable_refs(&tag_json);

    // Build variable index for resolution
    let all_variables: Vec<&Value> = variables_data
        .get("variable")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    let var_map: std::collections::HashMap<&str, &Value> = all_variables
        .iter()
        .filter_map(|v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .map(|name| (name, *v))
        })
        .collect();

    let variables: Vec<VarInfo> = referenced_vars
        .iter()
        .map(|name| {
            if let Some(v) = var_map.get(name.as_str()) {
                VarInfo {
                    name: name.clone(),
                    var_type: str_field(v, "type"),
                    id: str_field(v, "variableId"),
                }
            } else {
                VarInfo {
                    name: name.clone(),
                    var_type: "built-in".into(),
                    id: "-".into(),
                }
            }
        })
        .collect();

    // Extract key parameters
    let params = extract_key_params(&tag);

    match format {
        OutputFormat::Json => {
            let output = json!({
                "tag": {
                    "id": tag_id,
                    "name": tag_name,
                    "type": tag_type,
                },
                "parameters": params,
                "firingTriggers": firing.iter().map(|t| json!({
                    "id": t.id, "name": t.name, "type": t.trigger_type,
                })).collect::<Vec<_>>(),
                "blockingTriggers": blocking.iter().map(|t| json!({
                    "id": t.id, "name": t.name, "type": t.trigger_type,
                })).collect::<Vec<_>>(),
                "referencedVariables": variables.iter().map(|v| json!({
                    "name": v.name, "type": v.var_type, "id": v.id,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputFormat::Table | OutputFormat::Compact => {
            println!("Tag: {} (id: {}, type: {})", tag_name, tag_id, tag_type);

            if !params.is_empty() {
                println!("\nParameters:");
                for (k, v) in &params {
                    println!("  {k}: {v}");
                }
            }

            println!("\nFiring Triggers:");
            if firing.is_empty() {
                println!("  (none)");
            } else {
                for t in &firing {
                    println!("  [{}] {} ({})", t.id, t.name, t.trigger_type);
                }
            }

            if !blocking.is_empty() {
                println!("\nBlocking Triggers:");
                for t in &blocking {
                    println!("  [{}] {} ({})", t.id, t.name, t.trigger_type);
                }
            }

            println!("\nReferenced Variables:");
            if variables.is_empty() {
                println!("  (none)");
            } else {
                for v in &variables {
                    if v.id == "-" {
                        println!("  {{{{{}}}}} ({})", v.name, v.var_type);
                    } else {
                        println!("  {{{{{}}}}} (id: {}, type: {})", v.name, v.id, v.var_type);
                    }
                }
            }
        }
    }

    Ok(())
}

struct TriggerInfo {
    id: String,
    name: String,
    trigger_type: String,
}

struct VarInfo {
    name: String,
    var_type: String,
    id: String,
}

fn resolve_trigger(id: &str, map: &std::collections::HashMap<&str, &Value>) -> TriggerInfo {
    if let Some(t) = map.get(id) {
        TriggerInfo {
            id: id.into(),
            name: str_field(t, "name"),
            trigger_type: str_field(t, "type"),
        }
    } else {
        TriggerInfo {
            id: id.into(),
            name: "(unknown)".into(),
            trigger_type: "-".into(),
        }
    }
}

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("-")
        .to_string()
}

fn id_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_variable_refs(text: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i + 3 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = text[i + 2..].find("}}") {
                let name = &text[i + 2..i + 2 + end];
                // Skip internal GTM variables like _event
                if !name.starts_with('_') && !name.is_empty() && seen.insert(name.to_string()) {
                    vars.push(name.to_string());
                }
                i = i + 2 + end + 2;
                continue;
            }
        }
        i += 1;
    }
    vars
}

fn extract_key_params(tag: &Value) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let Some(params) = tag.get("parameter").and_then(|v| v.as_array()) {
        for p in params {
            let key = p.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let value = p.get("value").and_then(|v| v.as_str()).unwrap_or("");
            // Show important params, skip internal/long ones
            match key {
                "html" => {
                    let preview = if value.len() > 80 {
                        format!("{}... ({} chars)", &value[..80], value.len())
                    } else {
                        value.to_string()
                    };
                    result.push((key.to_string(), preview));
                }
                "" => {}
                _ => {
                    result.push((key.to_string(), value.to_string()));
                }
            }
        }
    }
    result
}

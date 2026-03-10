use clap::ValueEnum;
use serde_json::Value;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

pub fn print_output(value: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            );
        }
        OutputFormat::Table => {
            // TODO: table rendering per resource type
            // For now, fall back to JSON
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            );
        }
    }
}

pub fn print_deleted(resource: &str, id: &str) {
    let msg = serde_json::json!({
        "status": "deleted",
        "resource": resource,
        "id": id,
    });
    println!("{}", serde_json::to_string_pretty(&msg).unwrap());
}

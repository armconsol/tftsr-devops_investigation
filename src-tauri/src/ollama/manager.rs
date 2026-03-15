use crate::ollama::OllamaModel;
use tauri::Emitter;

const OLLAMA_BASE: &str = "http://localhost:11434";

pub async fn list_models() -> anyhow::Result<Vec<OllamaModel>> {
    let resp: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/api/tags", OLLAMA_BASE))
        .send()
        .await?
        .json()
        .await?;

    let models = resp["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    Some(OllamaModel {
                        name: m["name"].as_str()?.to_string(),
                        size: m["size"].as_u64().unwrap_or(0),
                        modified: m["modified_at"].as_str().unwrap_or("").to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

pub async fn pull_model(app_handle: tauri::AppHandle, model_name: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut resp = client
        .post(format!("{}/api/pull", OLLAMA_BASE))
        .json(&serde_json::json!({ "name": model_name, "stream": true }))
        .send()
        .await?;

    while let Some(chunk) = resp.chunk().await? {
        if let Ok(s) = String::from_utf8(chunk.to_vec()) {
            for line in s.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    let status = json["status"].as_str().unwrap_or("").to_string();
                    let total = json["total"].as_u64().unwrap_or(0);
                    let completed = json["completed"].as_u64().unwrap_or(0);
                    let percent = if total > 0 {
                        (completed as f64 / total as f64 * 100.0) as u32
                    } else {
                        0
                    };

                    let _ = app_handle.emit(
                        "model://progress",
                        serde_json::json!({
                            "name": model_name,
                            "status": status,
                            "total": total,
                            "completed": completed,
                            "percent": percent,
                        }),
                    );
                }
            }
        }
    }
    Ok(())
}

pub async fn delete_model(model_name: &str) -> anyhow::Result<()> {
    let resp = reqwest::Client::new()
        .delete(format!("{}/api/delete", OLLAMA_BASE))
        .json(&serde_json::json!({ "name": model_name }))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to delete model: {}", resp.status());
    }
    Ok(())
}

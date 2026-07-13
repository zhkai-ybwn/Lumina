use serde_json::json;

use crate::git::models::{AiModelConfig, GitAiAnalysis, OllamaGenerateResponse};
use crate::git::prompt::build_analysis_schema;

pub async fn call_ollama_generate(config: &AiModelConfig, prompt: &str) -> Result<GitAiAnalysis, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let request_body = json!({
        "model": config.model,
        "prompt": prompt,
        "stream": false,
        "format": build_analysis_schema("en"),
        "think": false,
        "options": {
            "temperature": 0.2
        }
    });

    let response = client
        .post(format!("{}/api/generate", config.base_url.trim_end_matches('/')))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            format!(
                "请求 Ollama 失败: {}。请确认 Ollama 正在运行，并且模型 {} 已可用。",
                e, config.model
            )
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误响应".to_string());
        return Err(format!("Ollama 返回错误状态 {}: {}", status, text));
    }

    let ollama_resp: OllamaGenerateResponse = response
        .json()
        .await
        .map_err(|e| format!("解析 Ollama 响应失败: {}", e))?;

    if let Some(err) = ollama_resp.error {
        return Err(format!("Ollama 返回错误: {}", err));
    }

    let cleaned = ollama_resp.response.trim();

    let parsed: GitAiAnalysis = serde_json::from_str(cleaned).map_err(|e| {
        format!(
            "模型返回的 JSON 解析失败: {}。原始返回片段: {}",
            e,
            cleaned.chars().take(500).collect::<String>()
        )
    })?;

    Ok(parsed)
}

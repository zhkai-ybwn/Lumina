use crate::ai::ollama;
use crate::ai::qwencloud;
use crate::git::models::{
    AiModelConfig, AiProviderType, GitAiAnalysis, GitAiPayload, GitCommitPayload,
    GitCommitPromptPayload, GitCommitPromptPreview, GitFileDiffPayload, GitFileDiffResponse,
    GitCommandResult, GitPromptAiPayload, GitPullPayload, GitPushPayload, GitSnapshot,
};
use crate::git::prompt::{build_analysis_prompt, build_selected_commit_prompt};
use crate::git::profile::{self, GitProjectProfileFile};
use crate::git::runner;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGitProjectProfilePayload {
    pub repo_path: String,
    pub content: String,
}

#[tauri::command]
pub fn load_git_snapshot(repo_path: String) -> Result<GitSnapshot, String> {
    runner::load_git_snapshot(&repo_path)
}

#[tauri::command]
pub fn load_git_file_diff(payload: GitFileDiffPayload) -> Result<GitFileDiffResponse, String> {
    runner::load_file_diff(&payload.repo_path, &payload.file_path, payload.staged)
}

#[tauri::command]
pub fn commit_git_changes(payload: GitCommitPayload) -> Result<GitCommandResult, String> {
    runner::commit_changes(&payload)
}

#[tauri::command]
pub fn push_git_changes(payload: GitPushPayload) -> Result<GitCommandResult, String> {
    runner::push_changes(&payload)
}

#[tauri::command]
pub fn pull_git_changes(payload: GitPullPayload) -> Result<GitCommandResult, String> {
    runner::pull_changes(&payload)
}

#[tauri::command]
pub fn ensure_git_project_profile(repo_path: String) -> Result<GitProjectProfileFile, String> {
    profile::ensure_project_profile(&repo_path)
}

#[tauri::command]
pub fn load_git_project_profile(repo_path: String) -> Result<GitProjectProfileFile, String> {
    profile::load_project_profile(&repo_path)
}

#[tauri::command]
pub fn save_git_project_profile(payload: SaveGitProjectProfilePayload) -> Result<GitProjectProfileFile, String> {
    profile::save_project_profile(&payload.repo_path, &payload.content)
}

#[tauri::command]
pub async fn build_git_commit_prompt(payload: GitCommitPromptPayload) -> Result<GitCommitPromptPreview, String> {
    tokio::task::spawn_blocking(move || {
        build_selected_commit_prompt(&payload.repo_path, &payload.branch, &payload.selected_files)
    })
    .await
    .map_err(|e| format!("构建 Commit Prompt 任务失败: {}", e))?
}

#[tauri::command]
pub async fn generate_git_ai_analysis(payload: GitAiPayload) -> Result<GitAiAnalysis, String> {
    let prompt = build_analysis_prompt(&payload);
    call_ai_with_prompt(&payload.model, &prompt).await
}

#[tauri::command]
pub async fn generate_git_ai_analysis_from_prompt(payload: GitPromptAiPayload) -> Result<GitAiAnalysis, String> {
    call_ai_with_prompt(&payload.model, &payload.prompt).await
}

async fn call_ai_with_prompt(model: &AiModelConfig, prompt: &str) -> Result<GitAiAnalysis, String> {
    match model.provider {
        AiProviderType::Ollama => ollama::call_ollama_generate(model, prompt).await,
        AiProviderType::OpenaiCompatible => {
            qwencloud::call_openai_compatible_generate(model, prompt).await
        }
    }
}

#[tauri::command]
pub async fn test_ai_model_connection(model: AiModelConfig) -> Result<String, String> {
    if !model.enabled {
        return Err("Model is disabled".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = match model.provider {
        AiProviderType::Ollama => {
            client
                .get(format!("{}/api/tags", model.base_url.trim_end_matches('/')))
                .send()
                .await
        }
        AiProviderType::OpenaiCompatible => {
            let mut request = client.get(format!("{}/models", model.base_url.trim_end_matches('/')));
            if let Some(api_key) = model.api_key.as_ref().map(|key| key.trim()).filter(|key| !key.is_empty()) {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }
            request.send().await
        }
    }
    .map_err(|e| format!("Connection failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Connection failed with status {}: {}", status, text));
    }

    Ok(format!("{} connected", model.name))
}

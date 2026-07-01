use std::fs;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AiProviderType {
    OpenaiCompatible,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModelConfig {
    pub id: String,
    pub name: String,
    pub provider: AiProviderType,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub default_model_id: String,
    pub models: Vec<AiModelConfig>,
    pub task_model_map: HashMap<String, String>,
}

const AI_SETTINGS_FILE: &str = "ai-settings.json";

fn ai_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| format!("解析应用配置目录失败: {}", e))?
        .join(AI_SETTINGS_FILE))
}

#[tauri::command]
pub fn load_ai_settings(app: AppHandle) -> Result<Option<AiSettings>, String> {
    let path = ai_settings_path(&app)?;
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 AI 设置失败 {}: {}", path.display(), e))?;
    let settings = serde_json::from_str::<AiSettings>(&content)
        .map_err(|e| format!("AI 设置文件不是合法 JSON {}: {}", path.display(), e))?;

    Ok(Some(settings))
}

#[tauri::command]
pub fn save_ai_settings(app: AppHandle, settings: AiSettings) -> Result<(), String> {
    let path = ai_settings_path(&app)?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("无法解析 AI 设置目录: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("创建 AI 设置目录失败 {}: {}", parent.display(), e))?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("序列化 AI 设置失败: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("写入 AI 设置失败 {}: {}", path.display(), e))
}

use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub project_path: String,
    pub package_json_path: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub package_manager: String,
    pub scripts: Vec<ProjectScript>,
    pub dependencies_count: usize,
    pub dev_dependencies_count: usize,
    pub detected_stack: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScript {
    pub name: String,
    pub command: String,
}

#[tauri::command]
pub async fn load_project_manifest(project_path: String) -> Result<ProjectManifest, String> {
    tokio::task::spawn_blocking(move || read_project_manifest(&project_path))
        .await
        .map_err(|e| format!("加载项目配置任务异常: {}", e))?
}

pub(crate) fn read_project_manifest(project_path: &str) -> Result<ProjectManifest, String> {
    let project_dir = Path::new(project_path);
    if !project_dir.is_dir() {
        return Err("请选择有效的项目目录。".to_string());
    }

    let package_json_path = project_dir.join("package.json");
    if !package_json_path.is_file() {
        return Err("当前目录未找到 package.json。".to_string());
    }

    let content = fs::read_to_string(&package_json_path)
        .map_err(|e| format!("读取 package.json 失败: {}", e))?;
    let package_json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("package.json 不是合法 JSON: {}", e))?;

    let scripts = package_json
        .get("scripts")
        .and_then(Value::as_object)
        .map(|script_map| {
            let mut scripts = script_map
                .iter()
                .filter_map(|(name, command)| {
                    command.as_str().map(|command| ProjectScript {
                        name: name.to_string(),
                        command: command.to_string(),
                    })
                })
                .collect::<Vec<_>>();
            scripts.sort_by(|left, right| left.name.cmp(&right.name));
            scripts
        })
        .unwrap_or_default();

    let dependencies_count = object_len(package_json.get("dependencies"));
    let dev_dependencies_count = object_len(package_json.get("devDependencies"));

    Ok(ProjectManifest {
        project_path: project_path.to_string(),
        package_json_path: package_json_path.to_string_lossy().to_string(),
        name: string_field(&package_json, "name"),
        version: string_field(&package_json, "version"),
        package_manager: detect_package_manager(project_dir, &package_json),
        scripts,
        dependencies_count,
        dev_dependencies_count,
        detected_stack: detect_stack(&package_json),
    })
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn object_len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_object).map(|object| object.len()).unwrap_or(0)
}

fn detect_package_manager(project_dir: &Path, package_json: &Value) -> String {
    if let Some(package_manager) = string_field(package_json, "packageManager") {
        return package_manager;
    }

    if project_dir.join("pnpm-lock.yaml").is_file() {
        return "pnpm".to_string();
    }
    if project_dir.join("yarn.lock").is_file() {
        return "yarn".to_string();
    }
    if project_dir.join("bun.lockb").is_file() || project_dir.join("bun.lock").is_file() {
        return "bun".to_string();
    }
    if project_dir.join("package-lock.json").is_file() {
        return "npm".to_string();
    }

    "npm".to_string()
}

fn detect_stack(package_json: &Value) -> Vec<String> {
    let mut stack = Vec::new();
    let has_dep = |name: &str| dependency_exists(package_json, name);

    for (dep, label) in [
        ("@tauri-apps/api", "Tauri"),
        ("vue", "Vue"),
        ("react", "React"),
        ("next", "Next.js"),
        ("vite", "Vite"),
        ("svelte", "Svelte"),
        ("typescript", "TypeScript"),
    ] {
        if has_dep(dep) {
            stack.push(label.to_string());
        }
    }

    stack
}

fn dependency_exists(package_json: &Value, name: &str) -> bool {
    ["dependencies", "devDependencies", "peerDependencies"]
        .iter()
        .any(|section| package_json.get(section).and_then(Value::as_object).is_some_and(|deps| deps.contains_key(name)))
}

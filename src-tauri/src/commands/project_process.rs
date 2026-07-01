use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader},
    net::TcpListener,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use tauri::State;

use super::project::read_project_manifest;

const LOG_LIMIT: usize = 500;
const PROCESS_LIMIT: usize = 40;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Default)]
pub struct ProjectProcessState {
    next_id: AtomicU64,
    processes: Mutex<HashMap<String, ManagedProcess>>,
}

impl Drop for ProjectProcessState {
    fn drop(&mut self) {
        if let Ok(processes) = self.processes.lock() {
            for process in processes.values() {
                if process_status(process).state == "running" {
                    kill_process_tree(process.meta.pid);
                    if let Ok(mut child) = process.child.lock() {
                        let _ = child.kill();
                    }
                }
            }
        }
    }
}

struct ManagedProcess {
    child: Arc<Mutex<Child>>,
    logs: Arc<Mutex<VecDeque<ProjectProcessLogLine>>>,
    status: Arc<Mutex<ProjectProcessStatus>>,
    meta: ProjectProcessMeta,
}

#[derive(Clone)]
struct ProjectProcessMeta {
    id: String,
    project_path: String,
    project_name: String,
    script_name: String,
    command: String,
    package_manager: String,
    pid: u32,
    started_at: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProcessSnapshot {
    pub id: String,
    pub project_path: String,
    pub project_name: String,
    pub script_name: String,
    pub command: String,
    pub package_manager: String,
    pub pid: u32,
    pub status: ProjectProcessStatus,
    pub started_at: u128,
    pub exited_at: Option<u128>,
    pub exit_code: Option<i32>,
    pub ports: Vec<u16>,
    pub log_count: usize,
    pub last_log_line: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProcessLogLine {
    pub stream: String,
    pub text: String,
    pub timestamp: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProcessLogs {
    pub process: ProjectProcessSnapshot,
    pub lines: Vec<ProjectProcessLogLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProcessStatus {
    pub state: String,
    pub exit_code: Option<i32>,
    pub exited_at: Option<u128>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartProjectProcessPayload {
    pub project_path: String,
    pub project_name: Option<String>,
    pub script_name: String,
    pub package_manager: String,
}

#[tauri::command]
pub async fn start_project_process(
    payload: StartProjectProcessPayload,
    state: State<'_, ProjectProcessState>,
) -> Result<ProjectProcessSnapshot, String> {
    start_process(payload, &state)
}

#[tauri::command]
pub async fn list_project_processes(
    state: State<'_, ProjectProcessState>,
) -> Result<Vec<ProjectProcessSnapshot>, String> {
    let mut processes = state
        .processes
        .lock()
        .map_err(|_| "读取进程列表失败。".to_string())?;
    let snapshots = processes.values().map(snapshot_process).collect::<Vec<_>>();
    processes.retain(|id, _| {
        snapshots
            .iter()
            .any(|process| process.id == *id && process.status.state == "running")
    });
    let mut snapshots = snapshots
        .into_iter()
        .filter(|process| process.status.state == "running")
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| right.started_at.cmp(&left.started_at));
    Ok(snapshots)
}

#[tauri::command]
pub async fn stop_project_process(
    process_id: String,
    state: State<'_, ProjectProcessState>,
) -> Result<ProjectProcessSnapshot, String> {
    let process = find_process(&state, &process_id)?;
    stop_process(&process)?;
    let snapshot = snapshot_process(&process);
    remove_process(&state, &process_id)?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn restart_project_process(
    process_id: String,
    state: State<'_, ProjectProcessState>,
) -> Result<ProjectProcessSnapshot, String> {
    let process = find_process(&state, &process_id)?;
    let meta = process.meta.clone();
    stop_process(&process)?;
    remove_process(&state, &process_id)?;
    let payload = StartProjectProcessPayload {
        project_path: meta.project_path,
        project_name: Some(meta.project_name),
        script_name: meta.script_name,
        package_manager: meta.package_manager,
    };
    start_process(payload, &state)
}

#[tauri::command]
pub async fn load_project_process_logs(
    process_id: String,
    state: State<'_, ProjectProcessState>,
) -> Result<ProjectProcessLogs, String> {
    let process = find_process(&state, &process_id)?;
    let lines = process
        .logs
        .lock()
        .map_err(|_| "读取进程日志失败。".to_string())?
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    Ok(ProjectProcessLogs {
        process: snapshot_process(&process),
        lines,
    })
}

#[tauri::command]
pub async fn open_project_url(url: String) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("只支持打开 http 或 https 链接。".to_string());
    }

    open_url(&url)
}

fn start_process(
    payload: StartProjectProcessPayload,
    state: &ProjectProcessState,
) -> Result<ProjectProcessSnapshot, String> {
    if !is_safe_script_name(&payload.script_name) {
        return Err("脚本名称包含不支持的字符。".to_string());
    }

    if let Some(existing) = find_active_script(state, &payload.project_path, &payload.script_name)? {
        return Ok(snapshot_process(&existing));
    }

    let manifest = read_project_manifest(&payload.project_path)?;
    let script = manifest
        .scripts
        .iter()
        .find(|script| script.name == payload.script_name)
        .ok_or_else(|| "package.json 中未找到该脚本。".to_string())?;
    let package_manager = package_manager_command(&payload.package_manager);
    let command_label = format!("{} run {}", package_manager, script.name);
    let project_name = payload
        .project_name
        .or(manifest.name)
        .unwrap_or_else(|| display_name_from_path(&payload.project_path));

    let mut command = shell_command(&command_label);
    command
        .current_dir(&payload.project_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| format!("启动命令失败: {}", e))?;
    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child = Arc::new(Mutex::new(child));
    let logs = Arc::new(Mutex::new(VecDeque::with_capacity(LOG_LIMIT)));
    let status = Arc::new(Mutex::new(ProjectProcessStatus {
        state: "running".to_string(),
        exit_code: None,
        exited_at: None,
    }));
    let id = format!("proc-{}-{}", now_millis(), state.next_id.fetch_add(1, Ordering::Relaxed));
    let meta = ProjectProcessMeta {
        id,
        project_path: payload.project_path,
        project_name,
        script_name: script.name.clone(),
        command: command_label,
        package_manager,
        pid,
        started_at: now_millis(),
    };

    if let Some(stdout) = stdout {
        spawn_log_reader(stdout, "stdout", Arc::clone(&logs));
    }
    if let Some(stderr) = stderr {
        spawn_log_reader(stderr, "stderr", Arc::clone(&logs));
    }
    spawn_waiter(Arc::clone(&child), Arc::clone(&status));

    let managed = ManagedProcess {
        child,
        logs,
        status,
        meta,
    };
    let snapshot = snapshot_process(&managed);
    let mut processes = state
        .processes
        .lock()
        .map_err(|_| "保存进程状态失败。".to_string())?;
    processes.insert(snapshot.id.clone(), managed);
    prune_finished_processes(&mut processes);
    Ok(snapshot)
}

fn find_process(state: &ProjectProcessState, process_id: &str) -> Result<ManagedProcess, String> {
    let processes = state
        .processes
        .lock()
        .map_err(|_| "读取进程状态失败。".to_string())?;
    processes
        .get(process_id)
        .map(clone_process)
        .ok_or_else(|| "进程不存在或已被清理。".to_string())
}

fn remove_process(state: &ProjectProcessState, process_id: &str) -> Result<(), String> {
    state
        .processes
        .lock()
        .map_err(|_| "清理进程状态失败。".to_string())?
        .remove(process_id);
    Ok(())
}

fn find_active_script(
    state: &ProjectProcessState,
    project_path: &str,
    script_name: &str,
) -> Result<Option<ManagedProcess>, String> {
    let processes = state
        .processes
        .lock()
        .map_err(|_| "读取进程状态失败。".to_string())?;
    Ok(processes
        .values()
        .find(|process| process.meta.project_path == project_path && process.meta.script_name == script_name && process_is_active(process))
        .map(clone_process))
}

fn stop_process(process: &ManagedProcess) -> Result<(), String> {
    let ports = process_ports(process);
    if process_status(process).state != "running" && !ports.iter().any(|port| is_port_listening(*port)) {
        return Ok(());
    }

    let pid = process.meta.pid;
    kill_process_tree(pid);
    kill_port_processes(&ports);
    if let Ok(mut child) = process.child.lock() {
        let _ = child.kill();
    }
    if let Ok(mut status) = process.status.lock() {
        status.state = "stopped".to_string();
        status.exited_at = Some(now_millis());
    }
    Ok(())
}

fn snapshot_process(process: &ManagedProcess) -> ProjectProcessSnapshot {
    let logs = process.logs.lock().ok();
    let ports = logs
        .as_ref()
        .map(|lines| detect_ports(lines))
        .unwrap_or_default();
    let mut status = process_status(process);
    if status.state == "exited" && ports.iter().any(|port| is_port_listening(*port)) {
        status.state = "running".to_string();
        status.exit_code = None;
        status.exited_at = None;
    }
    ProjectProcessSnapshot {
        id: process.meta.id.clone(),
        project_path: process.meta.project_path.clone(),
        project_name: process.meta.project_name.clone(),
        script_name: process.meta.script_name.clone(),
        command: process.meta.command.clone(),
        package_manager: process.meta.package_manager.clone(),
        pid: process.meta.pid,
        status: status.clone(),
        started_at: process.meta.started_at,
        exited_at: status.exited_at,
        exit_code: status.exit_code,
        ports,
        log_count: logs.as_ref().map(|lines| lines.len()).unwrap_or(0),
        last_log_line: logs
            .as_ref()
            .and_then(|lines| lines.back())
            .map(|line| line.text.clone()),
    }
}

fn detect_ports(lines: &VecDeque<ProjectProcessLogLine>) -> Vec<u16> {
    let mut ports = Vec::new();
    for line in lines.iter().rev().take(80) {
        let text = strip_ansi(&line.text);
        for token in text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != ':' && ch != '.' && ch != '/') {
            if let Some(port) = parse_port_token(token) {
                if !ports.contains(&port) {
                    ports.push(port);
                }
            }
        }
        if ports.len() >= 4 {
            break;
        }
    }
    ports.sort_unstable();
    ports
}

fn process_ports(process: &ManagedProcess) -> Vec<u16> {
    process
        .logs
        .lock()
        .map(|lines| detect_ports(&lines))
        .unwrap_or_default()
}

fn process_is_active(process: &ManagedProcess) -> bool {
    let status = process_status(process);
    status.state == "running" || process_ports(process).iter().any(|port| is_port_listening(*port))
}

fn strip_ansi(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn parse_port_token(token: &str) -> Option<u16> {
    let port_text = if let Some(index) = token.rfind(':') {
        &token[index + 1..]
    } else {
        return None;
    };
    let port = port_text.trim_matches('/').parse::<u16>().ok()?;
    if (1024..=65535).contains(&port) {
        Some(port)
    } else {
        None
    }
}

fn clone_process(process: &ManagedProcess) -> ManagedProcess {
    ManagedProcess {
        child: Arc::clone(&process.child),
        logs: Arc::clone(&process.logs),
        status: Arc::clone(&process.status),
        meta: process.meta.clone(),
    }
}

fn process_status(process: &ManagedProcess) -> ProjectProcessStatus {
    process.status.lock().map(|status| status.clone()).unwrap_or(ProjectProcessStatus {
        state: "unknown".to_string(),
        exit_code: None,
        exited_at: None,
    })
}

fn prune_finished_processes(processes: &mut HashMap<String, ManagedProcess>) {
    while processes.len() > PROCESS_LIMIT {
        let removable_id = processes
            .values()
            .filter(|process| process_status(process).state != "running")
            .min_by_key(|process| process.meta.started_at)
            .map(|process| process.meta.id.clone());
        if let Some(id) = removable_id {
            processes.remove(&id);
        } else {
            break;
        }
    }
}

fn spawn_log_reader<R>(reader: R, stream: &'static str, logs: Arc<Mutex<VecDeque<ProjectProcessLogLine>>>)
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            push_log(
                &logs,
                ProjectProcessLogLine {
                    stream: stream.to_string(),
                    text: line,
                    timestamp: now_millis(),
                },
            );
        }
    });
}

fn spawn_waiter(child: Arc<Mutex<Child>>, status: Arc<Mutex<ProjectProcessStatus>>) {
    thread::spawn(move || loop {
        let result = child.lock().ok().and_then(|mut child| child.try_wait().ok()).flatten();
        if let Some(exit_status) = result {
            if let Ok(mut status) = status.lock() {
                if status.state == "running" {
                    status.state = "exited".to_string();
                }
                status.exit_code = exit_status.code();
                status.exited_at = Some(now_millis());
            }
            break;
        }
        if status.lock().map(|status| status.state != "running").unwrap_or(true) {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    });
}

fn push_log(logs: &Arc<Mutex<VecDeque<ProjectProcessLogLine>>>, line: ProjectProcessLogLine) {
    if let Ok(mut logs) = logs.lock() {
        if logs.len() >= LOG_LIMIT {
            logs.pop_front();
        }
        logs.push_back(line);
    }
}

fn shell_command(command_line: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut command = silent_command("cmd");
        command.args(["/C", command_line]);
        command
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut command = Command::new("sh");
        command.args(["-c", command_line]);
        command
    }
}

fn silent_command(program: &str) -> Command {
    let mut command = Command::new(program);
    hide_command_window(&mut command);
    command
}

#[cfg(windows)]
fn hide_command_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_command_window(_command: &mut Command) {}

fn kill_process_tree(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = silent_command("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        silent_command("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
    }
    Ok(())
}

fn kill_port_processes(ports: &[u16]) {
    #[cfg(target_os = "windows")]
    {
        for port in ports {
            for pid in find_port_pids(*port) {
                let _ = silent_command("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_port_processes(_ports: &[u16]) {}

#[cfg(target_os = "windows")]
fn find_port_pids(port: u16) -> Vec<u32> {
    let output = silent_command("netstat").args(["-ano", "-p", "tcp"]).output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.contains("LISTENING") && line.contains(&format!(":{}", port)))
        .filter_map(|line| line.split_whitespace().last()?.parse::<u32>().ok())
        .collect()
}

fn is_port_listening(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err() || TcpListener::bind(("::1", port)).is_err()
}

fn package_manager_command(package_manager: &str) -> String {
    if matches!(package_manager, "corepack pnpm" | "corepack yarn") {
        return package_manager.to_string();
    }
    let name = package_manager.split('@').next().unwrap_or(package_manager).trim();
    let has_version = package_manager.contains('@');
    match (name, has_version) {
        ("pnpm", true) => "corepack pnpm",
        ("yarn", true) => "corepack yarn",
        ("pnpm", false) => "pnpm",
        ("yarn", false) => "yarn",
        ("bun", _) => "bun",
        _ => "npm",
    }
    .to_string()
}

fn is_safe_script_name(script_name: &str) -> bool {
    !script_name.is_empty()
        && script_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '.'))
}

fn display_name_from_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .last()
        .unwrap_or(path)
        .to_string()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

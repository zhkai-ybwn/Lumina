use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::git::models::{
    GitCommandResult, GitCommitChangedFile, GitCommitDetail, GitCommitDetailPayload, GitCommitFileDiffPayload,
    GitCommitFileDiffResponse, GitCommitPayload, GitConfigureRemotePayload, GitFileActionPayload,
    GitFileDiffResponse, GitFileStat, GitFilesActionPayload, GitLogEntry, GitLogPayload, GitPullPayload,
    GitPushPayload, GitRebasePayload, GitRepoPayload, GitRepairUpstreamPayload, GitRepositoryState, GitSnapshot,
    GitSyncRecommendedAction, GitSyncStatus,
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let output = git_command(repo_path)
        .args(args)
        .output()
        .map_err(|e| format!("执行 git 命令失败 {:?}: {}", args, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("git {:?} 执行失败", args)
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn run_git_raw(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let output = git_command(repo_path)
        .args(args)
        .output()
        .map_err(|e| format!("执行 git 命令失败 {:?}: {}", args, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("git {:?} 执行失败", args)
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_raw_owned(repo_path: &str, args: &[String]) -> Result<String, String> {
    let output = git_command(repo_path)
        .args(args)
        .output()
        .map_err(|e| format!("执行 git 命令失败 {:?}: {}", args, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("git {:?} 执行失败", args)
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_capture(repo_path: &str, args: &[&str]) -> Result<GitCommandResult, String> {
    let output = git_command(repo_path)
        .args(args)
        .output()
        .map_err(|e| format!("执行 git 命令失败 {:?}: {}", args, e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let command = format!("git {}", args.join(" "));

    if !output.status.success() {
        let suggestion = git_error_suggestion(&stderr);
        return Err(format_command_error(&command, &stdout, &stderr, suggestion.as_deref()));
    }

    Ok(GitCommandResult {
        command,
        message: if stdout.trim().is_empty() {
            "Command completed".to_string()
        } else {
            stdout.trim().to_string()
        },
        stdout,
        stderr,
        suggestion: None,
    })
}

fn run_git_capture_status(repo_path: &str, args: &[&str]) -> Result<(bool, GitCommandResult), String> {
    let output = git_command(repo_path)
        .args(args)
        .output()
        .map_err(|e| format!("执行 git 命令失败 {:?}: {}", args, e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let command = format!("git {}", args.join(" "));
    let success = output.status.success();
    let message = if stdout.trim().is_empty() {
        if success {
            "Command completed".to_string()
        } else {
            "Command failed".to_string()
        }
    } else {
        stdout.trim().to_string()
    };

    Ok((
        success,
        GitCommandResult {
            command,
            message,
            stdout,
            stderr,
            suggestion: None,
        },
    ))
}

fn git_command(repo_path: &str) -> Command {
    let mut command = silent_command("git");
    command.current_dir(repo_path);
    apply_git_proxy_env(&mut command, repo_path);
    command
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

fn apply_git_proxy_env(command: &mut Command, repo_path: &str) {
    if should_bypass_proxy_for_repo(repo_path) {
        clear_proxy_env(command);
        return;
    }

    let proxy = resolve_git_proxy_env(repo_path);
    set_proxy_env_pair(command, "HTTP_PROXY", "http_proxy", proxy.http.as_deref());
    set_proxy_env_pair(command, "HTTPS_PROXY", "https_proxy", proxy.https.as_deref());
    set_proxy_env_pair(command, "ALL_PROXY", "all_proxy", proxy.all.as_deref());
}

fn clear_proxy_env(command: &mut Command) {
    for key in [
        "HTTP_PROXY",
        "http_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        command.env_remove(key);
    }
}

fn set_proxy_env_pair(command: &mut Command, upper: &str, lower: &str, value: Option<&str>) {
    if let Some(value) = value {
        command.env(upper, value);
        command.env(lower, value);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct GitProxyEnv {
    http: Option<String>,
    https: Option<String>,
    all: Option<String>,
}

fn resolve_git_proxy_env(_repo_path: &str) -> GitProxyEnv {
    let mut proxy = GitProxyEnv {
        http: env_proxy(&["HTTP_PROXY", "http_proxy"]),
        https: env_proxy(&["HTTPS_PROXY", "https_proxy"]),
        all: env_proxy(&["ALL_PROXY", "all_proxy"]),
    };

    if proxy.http.is_none() && proxy.https.is_none() && proxy.all.is_none() {
        if let Some(system_proxy) = windows_system_proxy() {
            proxy = system_proxy;
        }
    }

    if proxy.https.is_none() {
        proxy.https = proxy.http.clone().or_else(|| proxy.all.clone());
    }
    if proxy.http.is_none() {
        proxy.http = proxy.https.clone().or_else(|| proxy.all.clone());
    }
    if proxy.all.is_none() {
        proxy.all = proxy.https.clone().or_else(|| proxy.http.clone());
    }

    proxy
}

fn should_bypass_proxy_for_repo(repo_path: &str) -> bool {
    let Some(remote_url) = remote_origin_url(repo_path) else {
        return false;
    };
    let Some(host) = remote_host_from_url(&remote_url) else {
        return false;
    };

    is_proxy_bypass_host(&host) || host_matches_no_proxy(&host)
}

fn remote_origin_url(repo_path: &str) -> Option<String> {
    let output = silent_command("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn remote_host_from_url(remote_url: &str) -> Option<String> {
    let remote_url = remote_url.trim();
    if remote_url.is_empty() {
        return None;
    }

    if let Some((_, rest)) = remote_url.split_once("://") {
        let authority = rest.split(['/', '?', '#']).next()?.trim();
        let host_port = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
        return Some(strip_port(host_port)?.to_ascii_lowercase());
    }

    if let Some((user_host, _path)) = remote_url.split_once(':') {
        if let Some((_, host)) = user_host.rsplit_once('@') {
            return Some(host.trim().to_ascii_lowercase());
        }
    }

    None
}

fn strip_port(host_port: &str) -> Option<&str> {
    let host_port = host_port.trim();
    if host_port.is_empty() {
        return None;
    }
    if let Some(rest) = host_port.strip_prefix('[') {
        let (host, _) = rest.split_once(']')?;
        return Some(host);
    }

    Some(host_port.split(':').next().unwrap_or(host_port))
}

fn is_proxy_bypass_host(host: &str) -> bool {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return false;
    }
    if host == "localhost" || host.ends_with(".local") || !host.contains('.') {
        return true;
    }

    if let Ok(IpAddr::V4(ip)) = host.parse::<IpAddr>() {
        let octets = ip.octets();
        return octets[0] == 10
            || (octets[0] == 172 && (16..=31).contains(&octets[1]))
            || (octets[0] == 192 && octets[1] == 168)
            || octets[0] == 127
            || (octets[0] == 169 && octets[1] == 254);
    }

    false
}

fn host_matches_no_proxy(host: &str) -> bool {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return false;
    }

    let no_proxy = [
        std::env::var("NO_PROXY").ok(),
        std::env::var("no_proxy").ok(),
        windows_proxy_override(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(",");

    no_proxy
        .split([',', ';', ' '])
        .filter_map(|item| {
            let item = item.trim().trim_end_matches('.').to_ascii_lowercase();
            if item.is_empty() {
                None
            } else {
                Some(item)
            }
        })
        .any(|rule| no_proxy_rule_matches_host(&rule, &host))
}

fn no_proxy_rule_matches_host(rule: &str, host: &str) -> bool {
    if rule == "*" {
        return true;
    }
    if rule == "<local>" {
        return !host.contains('.');
    }
    if let Some(prefix) = rule.strip_suffix('*') {
        return host.starts_with(prefix);
    }
    if let Some(suffix) = rule.strip_prefix("*.") {
        return host == suffix || host.ends_with(&format!(".{}", suffix));
    }
    if let Some(suffix) = rule.strip_prefix('.') {
        return host == suffix || host.ends_with(&format!(".{}", suffix));
    }

    host == rule
}

fn env_proxy(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        std::env::var(key)
            .ok()
            .and_then(|value| normalize_http_proxy_value(&value))
    })
}

fn normalize_http_proxy_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.contains("://") {
        return Some(value.to_string());
    }

    Some(format!("http://{}", value))
}

fn normalize_socks_proxy_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.contains("://") {
        return Some(value.to_string());
    }

    Some(format!("socks5://{}", value))
}

#[cfg(windows)]
fn windows_system_proxy() -> Option<GitProxyEnv> {
    let output = silent_command("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let enabled = registry_value(&text, "ProxyEnable")
        .is_some_and(|value| value == "0x1" || value == "1");
    if !enabled {
        return None;
    }

    registry_value(&text, "ProxyServer")
        .and_then(|value| proxy_env_from_windows_proxy_server(&value))
}

#[cfg(windows)]
fn windows_proxy_override() -> Option<String> {
    let output = silent_command("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    registry_value(&text, "ProxyOverride")
}

#[cfg(not(windows))]
fn windows_proxy_override() -> Option<String> {
    None
}

#[cfg(not(windows))]
fn windows_system_proxy() -> Option<GitProxyEnv> {
    None
}

fn registry_value(text: &str, name: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with(name) {
            return None;
        }

        let mut parts = line.split_whitespace();
        let value_name = parts.next()?;
        if value_name != name {
            return None;
        }

        let _value_type = parts.next()?;
        let value = parts.collect::<Vec<_>>().join(" ");
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    })
}

fn proxy_env_from_windows_proxy_server(value: &str) -> Option<GitProxyEnv> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    if !value.contains('=') {
        let proxy = normalize_http_proxy_value(value)?;
        return Some(GitProxyEnv {
            http: Some(proxy.clone()),
            https: Some(proxy.clone()),
            all: Some(proxy),
        });
    }

    let mut proxy = GitProxyEnv::default();
    for part in value.split(';') {
        let Some((kind, address)) = part.split_once('=') else {
            continue;
        };
        let kind = kind.trim().to_ascii_lowercase();
        match kind.as_str() {
            "http" => proxy.http = normalize_http_proxy_value(address),
            "https" => proxy.https = normalize_http_proxy_value(address),
            "socks" => proxy.all = normalize_socks_proxy_value(address),
            _ => {}
        }
    }

    if proxy.https.is_none() {
        proxy.https = proxy.http.clone().or_else(|| proxy.all.clone());
    }
    if proxy.http.is_none() {
        proxy.http = proxy.https.clone().or_else(|| proxy.all.clone());
    }
    if proxy.all.is_none() {
        proxy.all = proxy.https.clone().or_else(|| proxy.http.clone());
    }

    if proxy.http.is_none() && proxy.https.is_none() && proxy.all.is_none() {
        None
    } else {
        Some(proxy)
    }
}

fn format_command_error(command: &str, stdout: &str, stderr: &str, suggestion: Option<&str>) -> String {
    let mut message = format!("> {}\n", command);
    if !stdout.trim().is_empty() {
        message.push_str(stdout.trim());
        message.push('\n');
    }
    if !stderr.trim().is_empty() {
        message.push_str(stderr.trim());
    }
    if let Some(suggestion) = suggestion {
        message.push_str("\n\n建议: ");
        message.push_str(suggestion);
    }
    message
}

fn git_error_suggestion(stderr: &str) -> Option<String> {
    let lower = stderr.to_lowercase();
    if lower.contains("ambiguous argument 'head'") || lower.contains("unknown revision") {
        Some("当前仓库还没有首个提交。请先完成 Commit，再使用 Push -u origin main 推送到 GitHub。".to_string())
    } else if lower.contains("no such ref was fetched") {
        Some("当前分支配置的 upstream 在远端不存在。请重新设置上游分支，或确认远端默认分支名称。".to_string())
    } else if lower.contains("no tracking information") || lower.contains("no upstream") {
        Some("当前分支没有 upstream。请先 push -u 设置上游分支，或选择远端分支。".to_string())
    } else if lower.contains("would be overwritten") {
        Some("本地改动会被覆盖。请先提交、暂存或撤销相关改动后再拉取。".to_string())
    } else if lower.contains("non-fast-forward") || lower.contains("fetch first") {
        Some("远端包含本地没有的提交。请先拉取远端变更，再重新推送。".to_string())
    } else if lower.contains("failed to connect")
        || lower.contains("could not connect to server")
        || lower.contains("connection timed out")
        || lower.contains("unable to access")
    {
        Some("Git 已进入网络连接阶段，但无法连接远端服务器。请检查 GitHub/GitLab 网络、代理/VPN、公司防火墙，或在命令行执行 git ls-remote <remote-url> 验证连通性。若命令行可连接但客户端失败，通常是客户端启动环境没有继承代理，可重启客户端或配置 Git 的 http.proxy/https.proxy。这个错误通常不是 user.name/email 导致的。".to_string())
    } else {
        None
    }
}

fn clear_index_for_selected_commit(repo_path: &str) -> Result<GitCommandResult, String> {
    if run_git(repo_path, &["rev-parse", "--verify", "HEAD"]).is_ok() {
        return run_git_capture(repo_path, &["reset"]);
    }

    run_git_capture(repo_path, &["read-tree", "--empty"])
}

fn validate_selected_files(repo_path: &str, selected_files: &[String]) -> Result<(), String> {
    for file in selected_files {
        let normalized = file.trim();
        if normalized.is_empty() {
            return Err("选中文件路径为空，请刷新仓库状态后重新勾选文件。".to_string());
        }

        let exists_in_worktree = Path::new(repo_path).join(normalized).exists();
        let tracked_by_git = run_git(repo_path, &["ls-files", "--error-unmatch", "--", normalized]).is_ok();
        let ignored_by_git = is_git_ignored(repo_path, normalized);
        if ignored_by_git && !tracked_by_git {
            return Err(format!(
                "选中文件已被 .gitignore 忽略，不能作为普通文件加入提交: {}\n\n建议: 请刷新仓库状态后重新勾选文件。",
                normalized
            ));
        }
        if !exists_in_worktree && !tracked_by_git {
            return Err(format!(
                "选中文件在当前仓库中不存在，也不是已跟踪文件: {}\n\n建议: 请刷新仓库状态后重新勾选文件。如果刚切换过项目，请确认当前仓库路径是否正确。",
                normalized
            ));
        }
    }

    Ok(())
}

fn is_git_ignored(repo_path: &str, file_path: &str) -> bool {
    git_command(repo_path)
        .args(["check-ignore", "--no-index", "--quiet", "--", file_path])
        .status()
        .is_ok_and(|status| status.success())
}

fn stage_selected_file_for_commit(repo_path: &str, file_path: &str) -> Result<GitCommandResult, String> {
    if is_git_ignored(repo_path, file_path)
        && run_git(repo_path, &["ls-files", "--error-unmatch", "--", file_path]).is_ok()
    {
        return run_git_capture(repo_path, &["rm", "--cached", "--", file_path]);
    }

    run_git_capture(repo_path, &["add", "--", file_path])
}

fn has_unmerged_files(repo_path: &str) -> bool {
    let Ok(status_raw) = run_git_raw(repo_path, &["status", "--porcelain=v1", "--untracked-files=all"]) else {
        return false;
    };

    status_raw.lines().any(|line| {
        let status = line.get(0..2).unwrap_or_default();
        matches!(status, "DD" | "AU" | "UD" | "UA" | "DU" | "AA" | "UU")
    })
}

fn git_path_exists(repo_path: &str, git_path: &str) -> bool {
    let Ok(relative_path) = run_git(repo_path, &["rev-parse", "--git-path", git_path]) else {
        return false;
    };

    Path::new(repo_path).join(relative_path.trim()).exists()
}

fn load_repository_state(repo_path: &str) -> GitRepositoryState {
    let has_commits = run_git(repo_path, &["rev-parse", "--verify", "HEAD"]).is_ok();
    let remote_url = run_git(repo_path, &["remote", "get-url", "origin"]).ok();
    let upstream = if has_commits {
        run_git(repo_path, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).ok()
    } else {
        None
    };
    let upstream_gone = upstream
        .as_ref()
        .is_some_and(|value| run_git(repo_path, &["rev-parse", "--verify", value]).is_err());

    let (ahead, behind) = if has_commits && upstream.is_some() && !upstream_gone {
        run_git(repo_path, &["rev-list", "--left-right", "--count", "HEAD...@{u}"])
            .ok()
            .and_then(|value| {
                let mut parts = value.split_whitespace();
                let ahead = parts.next()?.parse::<usize>().ok()?;
                let behind = parts.next()?.parse::<usize>().ok()?;
                Some((ahead, behind))
            })
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    GitRepositoryState {
        has_commits,
        remote_name: remote_url.as_ref().map(|_| "origin".to_string()),
        remote_url,
        upstream,
        upstream_gone,
        ahead,
        behind,
        merge_in_progress: git_path_exists(repo_path, "MERGE_HEAD"),
        rebase_in_progress: git_path_exists(repo_path, "rebase-merge")
            || git_path_exists(repo_path, "rebase-apply"),
    }
}

fn recommended_sync_action(state: &GitRepositoryState) -> GitSyncRecommendedAction {
    if !state.has_commits {
        return GitSyncRecommendedAction::None;
    }
    if state.remote_url.is_none() {
        return GitSyncRecommendedAction::ConfigureRemote;
    }
    if state.upstream.is_none() || state.upstream_gone {
        return GitSyncRecommendedAction::PublishBranch;
    }
    if state.ahead > 0 && state.behind > 0 {
        return GitSyncRecommendedAction::ResolveDivergence;
    }
    if state.behind > 0 {
        return GitSyncRecommendedAction::Pull;
    }
    if state.ahead > 0 {
        return GitSyncRecommendedAction::Push;
    }

    GitSyncRecommendedAction::None
}

fn sync_status_message(state: &GitRepositoryState, action: &GitSyncRecommendedAction) -> String {
    let upstream = state.upstream.as_deref().unwrap_or("未设置");
    let base = format!(
        "远端检查完成\nupstream: {}\nahead: {}\nbehind: {}",
        upstream, state.ahead, state.behind
    );

    let advice = match action {
        GitSyncRecommendedAction::Push => "本地有待推送提交，可以继续 Push。",
        GitSyncRecommendedAction::Pull => "远端有本地没有的提交，请先 Pull，再 Push。",
        GitSyncRecommendedAction::ResolveDivergence => {
            "本地和远端都有新提交，不能直接 Push。请先 Pull 或打开 Log 确认分叉后再处理。"
        }
        GitSyncRecommendedAction::ConfigureRemote => "当前仓库没有 origin remote，请先配置远端。",
        GitSyncRecommendedAction::PublishBranch => "当前分支没有可用 upstream，可以发布当前分支并设置 upstream。",
        GitSyncRecommendedAction::None => "本地与远端已同步。",
    };

    format!("{}\n{}", base, advice)
}

pub fn sync_status(payload: &GitRepoPayload) -> Result<GitSyncStatus, String> {
    let fetch = run_git_capture(&payload.repo_path, &["fetch", "--prune"])?;
    let state = load_repository_state(&payload.repo_path);
    let recommended_action = recommended_sync_action(&state);
    let message = sync_status_message(&state, &recommended_action);
    let suggestion = match recommended_action {
        GitSyncRecommendedAction::Pull => Some("远端有本地没有的提交。请先 Pull，同步后再 Push。".to_string()),
        GitSyncRecommendedAction::ResolveDivergence => Some("当前分支已分叉。请先 Pull 并处理可能的冲突，或打开 Log 确认差异。".to_string()),
        GitSyncRecommendedAction::ConfigureRemote => Some("当前仓库没有 origin remote，推送前需要先连接远端仓库。".to_string()),
        GitSyncRecommendedAction::PublishBranch => Some("当前分支没有 upstream。Push 会使用 -u 发布当前分支。".to_string()),
        _ => None,
    };

    Ok(GitSyncStatus {
        command: fetch.command,
        message,
        stdout: fetch.stdout,
        stderr: fetch.stderr,
        suggestion,
        state,
        recommended_action,
    })
}

pub fn load_git_snapshot(repo_path: &str) -> Result<GitSnapshot, String> {
    let repo_root = run_git(repo_path, &["rev-parse", "--show-toplevel"])?;
    let branch = run_git(repo_path, &["branch", "--show-current"])?;
    let repository_state = load_repository_state(repo_path);
    let status_raw = run_git_raw(
    repo_path,
    &["status", "--porcelain=v1", "--untracked-files=all"],
)?;
    let staged_files_raw = run_git(repo_path, &["diff", "--cached", "--name-only"])?;
    let staged_diff = run_git(
        repo_path,
        &["diff", "--cached", "--unified=2", "--no-color"],
    )?;
    let file_stats = load_file_stats(repo_path)?;

    let status = if status_raw.trim().is_empty() {
        vec![]
    } else {
        status_raw.lines().map(|s| s.to_string()).collect()
    };

    let staged_files = if staged_files_raw.is_empty() {
        vec![]
    } else {
        staged_files_raw.lines().map(|s| s.to_string()).collect()
    };

    Ok(GitSnapshot {
        repo_path: repo_path.to_string(),
        repo_root,
        branch,
        repository_state,
        status,
        staged_files,
        staged_diff,
        file_stats,
    })
}

fn load_file_stats(repo_path: &str) -> Result<Vec<GitFileStat>, String> {
    if run_git(repo_path, &["rev-parse", "--verify", "HEAD"]).is_err() {
        return Ok(vec![]);
    }

    let unstaged_raw = run_git_raw(repo_path, &["diff", "--numstat", "--"])?;
    let staged_raw = run_git_raw(repo_path, &["diff", "--cached", "--numstat", "--"])?;

    let mut stats = unstaged_raw
        .lines()
        .chain(staged_raw.lines())
        .filter_map(parse_numstat_line)
        .collect::<Vec<_>>();

    stats.sort_by(|left, right| left.path.cmp(&right.path));
    stats.dedup_by(|left, right| {
        if left.path != right.path {
            return false;
        }

        right.added = merge_line_count(left.added, right.added);
        right.removed = merge_line_count(left.removed, right.removed);
        true
    });

    Ok(stats)
}

fn merge_line_count(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn parse_numstat_line(line: &str) -> Option<GitFileStat> {
    let mut parts = line.split('\t');
    let added_raw = parts.next()?;
    let removed_raw = parts.next()?;
    let path = parts.next()?.to_string();

    Some(GitFileStat {
        path,
        added: added_raw.parse::<usize>().ok(),
        removed: removed_raw.parse::<usize>().ok(),
    })
}

pub fn load_file_diff(repo_path: &str, file_path: &str, staged: bool) -> Result<GitFileDiffResponse, String> {
    let args = if staged {
        vec!["diff", "--cached", "--unified=3", "--no-color", "--", file_path]
    } else {
        vec!["diff", "--unified=3", "--no-color", "--", file_path]
    };

    let diff = run_git_raw(repo_path, &args)?;

    Ok(GitFileDiffResponse {
        file_path: file_path.to_string(),
        staged,
        diff,
    })
}

pub fn load_file_head_diff(payload: &GitFileActionPayload) -> Result<GitFileDiffResponse, String> {
    let diff = load_selected_file_diff(&payload.repo_path, &payload.file_path)?;

    Ok(GitFileDiffResponse {
        file_path: payload.file_path.clone(),
        staged: false,
        diff,
    })
}

pub fn load_selected_file_diff(repo_path: &str, file_path: &str) -> Result<String, String> {
    let unstaged = run_git_raw(repo_path, &["diff", "--unified=3", "--no-color", "--", file_path])?;
    let staged = run_git_raw(repo_path, &["diff", "--cached", "--unified=3", "--no-color", "--", file_path])?;
    let mut result = String::new();

    if !staged.trim().is_empty() {
        result.push_str(&staged);
    }

    if !unstaged.trim().is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&unstaged);
    }

    if result.trim().is_empty() {
        return load_untracked_file_diff(repo_path, file_path);
    }

    Ok(result)
}

fn load_untracked_file_diff(repo_path: &str, file_path: &str) -> Result<String, String> {
    let output = silent_command("git")
        .args(["diff", "--no-index", "--unified=3", "--no-color", "--", "/dev/null", file_path])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("执行 git diff --no-index 失败 {}: {}", file_path, e))?;

    if output.status.success() || output.status.code() == Some(1) {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        format!("git diff --no-index 执行失败: {}", file_path)
    } else {
        stderr
    })
}

pub fn commit_changes(payload: &GitCommitPayload) -> Result<GitCommandResult, String> {
    let title = payload.title.trim();
    let body = payload.body.trim();

    if title.is_empty() {
        return Err("Commit title cannot be empty".to_string());
    }

    if payload.selected_files.is_empty() {
        return Err("No files selected for commit".to_string());
    }

    validate_selected_files(&payload.repo_path, &payload.selected_files)?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut commands = Vec::new();

    let clear_index = clear_index_for_selected_commit(&payload.repo_path)?;
    commands.push(clear_index.command);
    stdout.push_str(&clear_index.stdout);
    stderr.push_str(&clear_index.stderr);

    for file in &payload.selected_files {
        let add = stage_selected_file_for_commit(&payload.repo_path, file)?;
        commands.push(add.command);
        stdout.push_str(&add.stdout);
        stderr.push_str(&add.stderr);
    }

    let commit = if body.is_empty() {
        run_git_capture(&payload.repo_path, &["commit", "-m", title])?
    } else {
        run_git_capture(&payload.repo_path, &["commit", "-m", title, "-m", body])?
    };
    commands.push(commit.command);
    stdout.push_str(&commit.stdout);
    stderr.push_str(&commit.stderr);

    Ok(GitCommandResult {
        command: commands.join("\n"),
        message: if commit.message.trim().is_empty() {
            "Commit completed".to_string()
        } else {
            commit.message
        },
        stdout,
        stderr,
        suggestion: None,
    })
}

pub fn push_changes(payload: &GitPushPayload) -> Result<GitCommandResult, String> {
    let branch = run_git(&payload.repo_path, &["branch", "--show-current"])?;
    if branch.trim().is_empty() {
        return Err("Cannot push because current branch is detached".to_string());
    }
    if run_git(&payload.repo_path, &["rev-parse", "--verify", "HEAD"]).is_err() {
        return Err("当前仓库还没有首个提交，无法推送。\n\n建议: 请先提交勾选文件，再执行 Push。".to_string());
    }
    if run_git(&payload.repo_path, &["remote", "get-url", "origin"]).is_err() {
        return Err("当前仓库没有配置 origin remote，无法推送。\n\n建议: 请先添加 GitHub 仓库地址，例如 git remote add origin <url>。".to_string());
    }

    let fetch = run_git_capture(&payload.repo_path, &["fetch", "--prune"])?;
    let state = load_repository_state(&payload.repo_path);
    let action = recommended_sync_action(&state);
    if matches!(action, GitSyncRecommendedAction::Pull | GitSyncRecommendedAction::ResolveDivergence) {
        return Err(sync_status_message(&state, &action));
    }

    let upstream = run_git(&payload.repo_path, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]);
    let mut result = match upstream {
        Ok(value) if run_git(&payload.repo_path, &["rev-parse", "--verify", &value]).is_ok() => {
            run_git_capture(&payload.repo_path, &["push"])?
        }
        _ => run_git_capture(&payload.repo_path, &["push", "-u", "origin", branch.trim()])?,
    };
    result.command = format!("{}\n{}", fetch.command, result.command);
    result.stdout = format!("{}{}", fetch.stdout, result.stdout);
    result.stderr = format!("{}{}", fetch.stderr, result.stderr);
    let message = if result.stdout.trim().is_empty() {
        "Push completed".to_string()
    } else {
        result.stdout.trim().to_string()
    };

    Ok(GitCommandResult {
        message,
        ..result
    })
}

pub fn pull_changes(payload: &GitPullPayload) -> Result<GitCommandResult, String> {
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut commands = Vec::new();
    let upstream = run_git(&payload.repo_path, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .map_err(|e| format!("当前分支没有 upstream，无法拉取。\n{}\n\n建议: 请先推送并设置 upstream，或手动设置上游分支。", e))?;

    let fetch = run_git_capture(&payload.repo_path, &["fetch", "--prune"])?;
    commands.push(fetch.command);
    stdout.push_str(&fetch.stdout);
    stderr.push_str(&fetch.stderr);

    if run_git(&payload.repo_path, &["rev-parse", "--verify", &upstream]).is_err() {
        return Err(format!(
            "当前分支配置的 upstream 是 {}，但 fetch 后没有找到这个远端引用。\n\n建议: 请检查远端分支是否存在，或重新设置当前分支的 upstream。",
            upstream
        ));
    }

    let (merge_success, merge) =
        run_git_capture_status(&payload.repo_path, &["merge", "--no-edit", &upstream])?;
    commands.push(merge.command);
    stdout.push_str(&merge.stdout);
    stderr.push_str(&merge.stderr);

    if !merge_success {
        if has_unmerged_files(&payload.repo_path) {
            return Ok(GitCommandResult {
                command: commands.join("\n"),
                message: "Pull 已拉取远端变更，但合并产生冲突。请解决冲突后标记 resolved，再提交合并结果。".to_string(),
                stdout,
                stderr,
                suggestion: Some("远端代码已经进入本地工作区。请在冲突列表中打开文件解决冲突，然后标记已解决并完成提交。".to_string()),
            });
        }

        let suggestion = git_error_suggestion(&stderr);
        return Err(format_command_error(
            &commands.join("\n"),
            &stdout,
            &stderr,
            suggestion.as_deref(),
        ));
    }

    Ok(GitCommandResult {
        command: commands.join("\n"),
        message: if stdout.trim().is_empty() {
            "Already up to date".to_string()
        } else {
            stdout.trim().to_string()
        },
        stdout,
        stderr,
        suggestion: None,
    })
}

pub fn rebase_changes(payload: &GitRebasePayload) -> Result<GitCommandResult, String> {
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut commands = Vec::new();
    let upstream = run_git(
        &payload.repo_path,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )
    .map_err(|e| format!("当前分支没有 upstream，无法 rebase。\n{}\n\n建议: 请先推送并设置 upstream，或手动设置上游分支。", e))?;

    let fetch = run_git_capture(&payload.repo_path, &["fetch", "--prune"])?;
    commands.push(fetch.command);
    stdout.push_str(&fetch.stdout);
    stderr.push_str(&fetch.stderr);

    if run_git(&payload.repo_path, &["rev-parse", "--verify", &upstream]).is_err() {
        return Err(format!(
            "当前分支配置的 upstream 是 {}，但 fetch 后没有找到这个远端引用。\n\n建议: 请检查远端分支是否存在，或重新设置当前分支的 upstream。",
            upstream
        ));
    }

    let (rebase_success, rebase) = run_git_capture_status(&payload.repo_path, &["rebase", &upstream])?;
    commands.push(rebase.command);
    stdout.push_str(&rebase.stdout);
    stderr.push_str(&rebase.stderr);

    if !rebase_success {
        if has_unmerged_files(&payload.repo_path) || load_repository_state(&payload.repo_path).rebase_in_progress {
            return Ok(GitCommandResult {
                command: commands.join("\n"),
                message: "Rebase 已开始，但产生冲突。请解决冲突后标记 resolved，再继续 rebase。".to_string(),
                stdout,
                stderr,
                suggestion: Some("远端代码已经拉取，本地提交正在重放。请解决冲突后点击 Continue Rebase，或 Abort Rebase 回到 rebase 前状态。".to_string()),
            });
        }

        let suggestion = git_error_suggestion(&stderr);
        return Err(format_command_error(
            &commands.join("\n"),
            &stdout,
            &stderr,
            suggestion.as_deref(),
        ));
    }

    Ok(GitCommandResult {
        command: commands.join("\n"),
        message: if stdout.trim().is_empty() {
            "Rebase completed".to_string()
        } else {
            stdout.trim().to_string()
        },
        stdout,
        stderr,
        suggestion: None,
    })
}

pub fn continue_rebase(payload: &GitRepoPayload) -> Result<GitCommandResult, String> {
    let (success, result) = run_git_capture_status(&payload.repo_path, &["rebase", "--continue"])?;
    if success {
        return Ok(GitCommandResult {
            message: if result.stdout.trim().is_empty() {
                "Rebase continued".to_string()
            } else {
                result.stdout.trim().to_string()
            },
            ..result
        });
    }

    if has_unmerged_files(&payload.repo_path) || load_repository_state(&payload.repo_path).rebase_in_progress {
        return Ok(GitCommandResult {
            message: "Rebase 仍有冲突或待处理步骤。请继续解决冲突后再执行 Continue Rebase。".to_string(),
            suggestion: Some("如果不想继续 rebase，可以执行 Abort Rebase。".to_string()),
            ..result
        });
    }

    let suggestion = git_error_suggestion(&result.stderr);
    Err(format_command_error(
        &result.command,
        &result.stdout,
        &result.stderr,
        suggestion.as_deref(),
    ))
}

pub fn abort_rebase(payload: &GitRepoPayload) -> Result<GitCommandResult, String> {
    let result = run_git_capture(&payload.repo_path, &["rebase", "--abort"])?;

    Ok(GitCommandResult {
        message: "Rebase 已中止".to_string(),
        ..result
    })
}

pub fn continue_merge(payload: &GitRepoPayload) -> Result<GitCommandResult, String> {
    let result = run_git_capture(&payload.repo_path, &["commit", "--no-edit"])?;

    Ok(GitCommandResult {
        message: "Merge commit 已完成".to_string(),
        ..result
    })
}

pub fn fetch_changes(payload: &GitRepoPayload) -> Result<GitCommandResult, String> {
    let result = run_git_capture(&payload.repo_path, &["fetch", "--prune"])?;

    Ok(GitCommandResult {
        message: if result.stdout.trim().is_empty() {
            "Fetch completed".to_string()
        } else {
            result.stdout.trim().to_string()
        },
        ..result
    })
}

pub fn configure_origin_remote(payload: &GitConfigureRemotePayload) -> Result<GitCommandResult, String> {
    let remote_url = payload.remote_url.trim();
    if remote_url.is_empty() {
        return Err("Remote URL cannot be empty".to_string());
    }

    let result = if run_git(&payload.repo_path, &["remote", "get-url", "origin"]).is_ok() {
        run_git_capture(&payload.repo_path, &["remote", "set-url", "origin", remote_url])?
    } else {
        run_git_capture(&payload.repo_path, &["remote", "add", "origin", remote_url])?
    };

    Ok(GitCommandResult {
        message: format!("origin configured: {}", remote_url),
        ..result
    })
}

pub fn repair_upstream(payload: &GitRepairUpstreamPayload) -> Result<GitCommandResult, String> {
    let branch = run_git(&payload.repo_path, &["branch", "--show-current"])?;
    let branch = branch.trim();
    if branch.is_empty() {
        return Err("Cannot repair upstream because current branch is detached".to_string());
    }

    if run_git(&payload.repo_path, &["remote", "get-url", "origin"]).is_err() {
        return Err("当前仓库没有配置 origin remote，无法修复 upstream。\n\n建议: 请先添加 GitHub 仓库地址。".to_string());
    }

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut commands = Vec::new();

    let fetch = run_git_capture(&payload.repo_path, &["fetch", "--prune", "origin"])?;
    commands.push(fetch.command);
    stdout.push_str(&fetch.stdout);
    stderr.push_str(&fetch.stderr);

    let upstream = format!("origin/{}", branch);
    if run_git(&payload.repo_path, &["rev-parse", "--verify", &upstream]).is_err() {
        return Err(format!(
            "远端还没有 {} 分支，无法直接修复 upstream。\n\n建议: 请使用 Push 发布当前分支，系统会自动执行 git push -u origin {}。",
            branch, branch
        ));
    }

    let repair = run_git_capture(&payload.repo_path, &["branch", "--set-upstream-to", &upstream, branch])?;
    commands.push(repair.command);
    stdout.push_str(&repair.stdout);
    stderr.push_str(&repair.stderr);

    Ok(GitCommandResult {
        command: commands.join("\n"),
        message: format!("Upstream repaired: {}", upstream),
        stdout,
        stderr,
        suggestion: None,
    })
}

pub fn open_file_external(payload: &GitFileActionPayload) -> Result<GitCommandResult, String> {
    let file_path = payload.file_path.trim();
    if file_path.is_empty() {
        return Err("文件路径为空，无法打开外部编辑器。".to_string());
    }

    let target_path = Path::new(&payload.repo_path).join(file_path);
    if !target_path.exists() {
        return Err(format!("文件不存在，无法打开外部编辑器: {}", file_path));
    }

    let (program, mut command) = if cfg!(target_os = "windows") {
        ("explorer", Command::new("explorer"))
    } else if cfg!(target_os = "macos") {
        ("open", Command::new("open"))
    } else {
        ("xdg-open", Command::new("xdg-open"))
    };

    command
        .arg(&target_path)
        .spawn()
        .map_err(|e| format!("打开外部编辑器失败 {}: {}", file_path, e))?;

    Ok(GitCommandResult {
        command: format!("{} {}", program, target_path.display()),
        message: format!("已打开外部编辑器: {}", file_path),
        stdout: String::new(),
        stderr: String::new(),
        suggestion: None,
    })
}

pub fn mark_files_resolved(payload: &GitFilesActionPayload) -> Result<GitCommandResult, String> {
    let files = payload
        .file_paths
        .iter()
        .map(|file| file.trim())
        .filter(|file| !file.is_empty())
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Err("请选择需要标记为已解决的冲突文件。".to_string());
    }

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut commands = Vec::new();

    for file in files {
        let result = run_git_capture(&payload.repo_path, &["add", "-A", "--", file])?;
        commands.push(result.command);
        stdout.push_str(&result.stdout);
        stderr.push_str(&result.stderr);
    }

    Ok(GitCommandResult {
        command: commands.join("\n"),
        message: "冲突文件已标记为已解决".to_string(),
        stdout,
        stderr,
        suggestion: None,
    })
}

pub fn abort_merge(payload: &GitRepoPayload) -> Result<GitCommandResult, String> {
    let result = run_git_capture(&payload.repo_path, &["merge", "--abort"])?;

    Ok(GitCommandResult {
        message: "Merge 已中止".to_string(),
        ..result
    })
}

pub fn load_git_log(payload: &GitLogPayload) -> Result<Vec<GitLogEntry>, String> {
    if run_git(&payload.repo_path, &["rev-parse", "--verify", "HEAD"]).is_err() {
        return Ok(vec![]);
    }

    let mut args = vec![
        "log".to_string(),
        "--max-count=1000".to_string(),
        "--date=iso-strict".to_string(),
        "--pretty=format:%H%x1f%h%x1f%an%x1f%ae%x1f%ad%x1f%s".to_string(),
    ];

    if let Some(file_path) = payload
        .file_path
        .as_ref()
        .map(|file| file.trim())
        .filter(|file| !file.is_empty())
    {
        args.push("--follow".to_string());
        args.push("--".to_string());
        args.push(file_path.to_string());
    }

    let raw = run_git_raw_owned(&payload.repo_path, &args)?;

    Ok(raw
        .lines()
        .filter_map(parse_log_line)
        .collect::<Vec<_>>())
}

pub fn load_git_commit_detail(payload: &GitCommitDetailPayload) -> Result<GitCommitDetail, String> {
    let hash = payload.hash.trim();
    if hash.is_empty() {
        return Err("Commit hash cannot be empty".to_string());
    }

    let meta = run_git_raw(
        &payload.repo_path,
        &[
            "show",
            "-s",
            "--date=iso-strict",
            "--pretty=format:%H%x1f%h%x1f%an%x1f%ae%x1f%ad%x1f%s%x1f%b",
            hash,
        ],
    )?;
    let mut parts = meta.splitn(7, '\x1f');
    let changed_files_raw = run_git_raw(&payload.repo_path, &["show", "--format=", "--name-status", "-M", hash])?;
    let numstat_raw = run_git_raw(&payload.repo_path, &["show", "--format=", "--numstat", "-M", hash])?;
    let file_stats = parse_commit_numstat(&numstat_raw);
    let short_stat = run_git_raw(&payload.repo_path, &["show", "--format=", "--shortstat", hash])?;

    Ok(GitCommitDetail {
        hash: parts.next().unwrap_or_default().to_string(),
        short_hash: parts.next().unwrap_or_default().to_string(),
        author_name: parts.next().unwrap_or_default().to_string(),
        author_email: parts.next().unwrap_or_default().to_string(),
        date: parts.next().unwrap_or_default().to_string(),
        subject: parts.next().unwrap_or_default().to_string(),
        body: parts.next().unwrap_or_default().trim().to_string(),
        short_stat: short_stat.trim().to_string(),
        changed_files: changed_files_raw
            .lines()
            .filter_map(|line| parse_commit_changed_file(line, &file_stats))
            .collect::<Vec<_>>(),
    })
}

pub fn load_git_commit_file_diff(payload: &GitCommitFileDiffPayload) -> Result<GitCommitFileDiffResponse, String> {
    let hash = payload.hash.trim();
    let file_path = payload.file_path.trim();
    if hash.is_empty() {
        return Err("Commit hash cannot be empty".to_string());
    }
    if file_path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }

    let diff = run_git_raw(
        &payload.repo_path,
        &["show", "--format=", "--unified=3", "--no-color", hash, "--", file_path],
    )?;

    Ok(GitCommitFileDiffResponse {
        hash: hash.to_string(),
        file_path: file_path.to_string(),
        diff,
    })
}

fn parse_log_line(line: &str) -> Option<GitLogEntry> {
    let mut parts = line.splitn(6, '\x1f');
    Some(GitLogEntry {
        hash: parts.next()?.to_string(),
        short_hash: parts.next()?.to_string(),
        author_name: parts.next()?.to_string(),
        author_email: parts.next()?.to_string(),
        date: parts.next()?.to_string(),
        subject: parts.next()?.to_string(),
    })
}

fn parse_commit_numstat(raw: &str) -> HashMap<String, (Option<usize>, Option<usize>)> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let added = parse_optional_line_count(parts.next()?);
            let removed = parse_optional_line_count(parts.next()?);
            let path = parts.next()?.to_string();
            Some((path, (added, removed)))
        })
        .collect()
}

fn parse_optional_line_count(value: &str) -> Option<usize> {
    value.parse::<usize>().ok()
}

fn parse_commit_changed_file(
    line: &str,
    file_stats: &HashMap<String, (Option<usize>, Option<usize>)>,
) -> Option<GitCommitChangedFile> {
    let parts = line.split('\t').collect::<Vec<_>>();
    let status = parts.first()?.trim();
    if status.is_empty() {
        return None;
    }

    if status.starts_with('R') || status.starts_with('C') {
        let path = parts.get(2).unwrap_or(parts.get(1)?).to_string();
        let (added, removed) = file_stats.get(&path).copied().unwrap_or((None, None));
        return Some(GitCommitChangedFile {
            status: status.to_string(),
            original_path: parts.get(1).map(|path| (*path).to_string()),
            path,
            added,
            removed,
        });
    }

    let path = parts.get(1)?.to_string();
    let (added, removed) = file_stats.get(&path).copied().unwrap_or((None, None));
    Some(GitCommitChangedFile {
        status: status.to_string(),
        path,
        original_path: None,
        added,
        removed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo(name: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lumina-{}-{}-{}", name, std::process::id(), now));
        fs::create_dir_all(&path).expect("create temp repo");
        path
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(path, content).expect("write test file");
    }

    fn git(repo_path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .expect("run git");

        if !output.status.success() {
            panic!(
                "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn normalizes_plain_windows_proxy_server() {
        let proxy = proxy_env_from_windows_proxy_server("127.0.0.1:10808").expect("proxy");

        assert_eq!(proxy.http.as_deref(), Some("http://127.0.0.1:10808"));
        assert_eq!(proxy.https.as_deref(), Some("http://127.0.0.1:10808"));
        assert_eq!(proxy.all.as_deref(), Some("http://127.0.0.1:10808"));
    }

    #[test]
    fn parses_split_windows_proxy_server() {
        let proxy = proxy_env_from_windows_proxy_server(
            "http=127.0.0.1:7890;https=127.0.0.1:7891;socks=127.0.0.1:7892",
        )
        .expect("proxy");

        assert_eq!(proxy.http.as_deref(), Some("http://127.0.0.1:7890"));
        assert_eq!(proxy.https.as_deref(), Some("http://127.0.0.1:7891"));
        assert_eq!(proxy.all.as_deref(), Some("socks5://127.0.0.1:7892"));
    }

    #[test]
    fn parses_registry_values() {
        let registry_output = r#"
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings
    ProxyEnable    REG_DWORD    0x1
    ProxyServer    REG_SZ    127.0.0.1:10808
"#;

        assert_eq!(registry_value(registry_output, "ProxyEnable").as_deref(), Some("0x1"));
        assert_eq!(
            registry_value(registry_output, "ProxyServer").as_deref(),
            Some("127.0.0.1:10808")
        );
    }

    #[test]
    fn extracts_remote_hosts() {
        assert_eq!(
            remote_host_from_url("http://192.168.0.127:9980/AMI/frontenddeveloper/ami-simulator.git").as_deref(),
            Some("192.168.0.127")
        );
        assert_eq!(
            remote_host_from_url("https://github.com/zhkai-ybwn/Lumina.git").as_deref(),
            Some("github.com")
        );
        assert_eq!(
            remote_host_from_url("git@github.com:zhkai-ybwn/Lumina.git").as_deref(),
            Some("github.com")
        );
    }

    #[test]
    fn bypasses_private_git_hosts() {
        assert!(is_proxy_bypass_host("192.168.0.127"));
        assert!(is_proxy_bypass_host("172.17.182.113"));
        assert!(is_proxy_bypass_host("10.0.8.12"));
        assert!(is_proxy_bypass_host("localhost"));
        assert!(!is_proxy_bypass_host("github.com"));
    }

    #[test]
    fn matches_no_proxy_rules() {
        assert!(no_proxy_rule_matches_host("*.corp.local", "git.corp.local"));
        assert!(no_proxy_rule_matches_host(".example.com", "git.example.com"));
        assert!(no_proxy_rule_matches_host("192.168.*", "192.168.0.127"));
        assert!(no_proxy_rule_matches_host("<local>", "gitlab"));
        assert!(!no_proxy_rule_matches_host("<local>", "github.com"));
    }

    #[test]
    fn recommends_sync_actions_from_ahead_behind() {
        let mut state = GitRepositoryState {
            has_commits: true,
            remote_name: Some("origin".to_string()),
            remote_url: Some("http://example.test/repo.git".to_string()),
            upstream: Some("origin/main".to_string()),
            upstream_gone: false,
            ahead: 1,
            behind: 0,
            merge_in_progress: false,
            rebase_in_progress: false,
        };
        assert_eq!(recommended_sync_action(&state), GitSyncRecommendedAction::Push);

        state.ahead = 0;
        state.behind = 1;
        assert_eq!(recommended_sync_action(&state), GitSyncRecommendedAction::Pull);

        state.ahead = 1;
        state.behind = 1;
        assert_eq!(
            recommended_sync_action(&state),
            GitSyncRecommendedAction::ResolveDivergence
        );

        state.ahead = 0;
        state.behind = 0;
        assert_eq!(recommended_sync_action(&state), GitSyncRecommendedAction::None);
    }

    #[test]
    fn parses_commit_changed_file_rename() {
        let mut stats = HashMap::new();
        stats.insert("src/new.ts".to_string(), (Some(2), Some(1)));
        let parsed = parse_commit_changed_file("R100\tsrc/old.ts\tsrc/new.ts", &stats).expect("changed file");

        assert_eq!(parsed.status, "R100");
        assert_eq!(parsed.original_path.as_deref(), Some("src/old.ts"));
        assert_eq!(parsed.path, "src/new.ts");
        assert_eq!(parsed.added, Some(2));
        assert_eq!(parsed.removed, Some(1));
    }

    #[test]
    fn commit_removes_tracked_files_that_are_now_ignored() {
        let repo = temp_repo("ignored-tracked-file");
        git(&repo, &["init"]);
        git(&repo, &["config", "user.name", "Lumina Test"]);
        git(&repo, &["config", "user.email", "lumina@example.test"]);

        let lumina_file = repo.join(".lumina").join("commit-prompt-debug.json");
        write_file(&lumina_file, "{\"version\":1}\n");
        git(&repo, &["add", "--", ".lumina/commit-prompt-debug.json"]);
        git(&repo, &["commit", "-m", "chore: add lumina debug file"]);

        write_file(&repo.join(".gitignore"), ".lumina\n");
        write_file(&lumina_file, "{\"version\":2}\n");

        let result = commit_changes(&GitCommitPayload {
            repo_path: repo.to_string_lossy().to_string(),
            title: "chore: ignore local lumina data".to_string(),
            body: String::new(),
            selected_files: vec![
                ".gitignore".to_string(),
                ".lumina/commit-prompt-debug.json".to_string(),
            ],
        })
        .expect("commit ignored tracked file removal");

        assert!(result
            .command
            .contains("git rm --cached -- .lumina/commit-prompt-debug.json"));
        assert!(lumina_file.exists());

        let tracked = Command::new("git")
            .args(["ls-files", "--error-unmatch", "--", ".lumina/commit-prompt-debug.json"])
            .current_dir(&repo)
            .output()
            .expect("run git ls-files");
        assert!(!tracked.status.success());

        fs::remove_dir_all(&repo).expect("remove temp repo");
    }
}

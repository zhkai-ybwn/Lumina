use std::path::Path;
use std::process::Command;

use crate::git::models::{
    GitCommandResult, GitCommitPayload, GitConfigureRemotePayload, GitFileDiffResponse, GitFileStat, GitPullPayload,
    GitPushPayload, GitRepairUpstreamPayload, GitRepositoryState, GitSnapshot,
};

pub fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
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
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
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
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
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
        if !exists_in_worktree && !tracked_by_git {
            return Err(format!(
                "选中文件在当前仓库中不存在，也不是已跟踪文件: {}\n\n建议: 请刷新仓库状态后重新勾选文件。如果刚切换过项目，请确认当前仓库路径是否正确。",
                normalized
            ));
        }
    }

    Ok(())
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
    }
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
    let output = Command::new("git")
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
        let add = run_git_capture(&payload.repo_path, &["add", "--", file])?;
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

    let upstream = run_git(&payload.repo_path, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]);
    let result = match upstream {
        Ok(value) if run_git(&payload.repo_path, &["rev-parse", "--verify", &value]).is_ok() => {
            run_git_capture(&payload.repo_path, &["push"])?
        }
        _ => run_git_capture(&payload.repo_path, &["push", "-u", "origin", branch.trim()])?,
    };
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

    let merge = run_git_capture(&payload.repo_path, &["merge", "--ff-only", &upstream])?;
    commands.push(merge.command);
    stdout.push_str(&merge.stdout);
    stderr.push_str(&merge.stderr);

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

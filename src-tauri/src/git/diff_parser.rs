use crate::git::models::{DiffEvidence, FileAction, FileKind, FileSemanticInfo};

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn split_diff_by_file(staged_diff: &str) -> Vec<(String, String)> {
    let mut results: Vec<(String, String)> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in staged_diff.lines() {
        if line.starts_with("diff --git ") {
            if let Some(file_path) = current_file.take() {
                results.push((file_path, current_lines.join("\n")));
                current_lines.clear();
            }

            // 例子: diff --git a/path/to/file.ts b/path/to/file.ts
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let b_path = parts[3].trim_start_matches("b/");
                current_file = Some(normalize_path(b_path));
            } else {
                current_file = None;
            }
        }

        if current_file.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(file_path) = current_file.take() {
        results.push((file_path, current_lines.join("\n")));
    }

    results
}

fn is_noise_line(line: &str) -> bool {
    line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
        || line.starts_with("@@")
}

fn trim_diff_excerpt(diff_text: &str, max_lines: usize) -> String {
    let mut kept: Vec<String> = Vec::new();

    for line in diff_text.lines() {
        if is_noise_line(line) {
            continue;
        }

        if line.starts_with('+') || line.starts_with('-') {
            kept.push(line.to_string());
        }

        if kept.len() >= max_lines {
            break;
        }
    }

    if kept.is_empty() {
        return diff_text
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n");
    }

    kept.join("\n")
}

fn file_priority(info: &FileSemanticInfo) -> i32 {
    let mut score = 0;

    score += match info.kind {
        FileKind::Page | FileKind::Component | FileKind::Config => 4,
        FileKind::Service | FileKind::Model => 3,
        FileKind::Constants => 2,
        FileKind::Route | FileKind::I18n => 1,
        FileKind::Style | FileKind::Test | FileKind::Unknown => 0,
    };

    score += match info.action {
        FileAction::Add => 3,
        FileAction::Update => 2,
        FileAction::Remove => 1,
        FileAction::Rename => 1,
    };

    score
}

fn same_file_path(a: &str, b: &str) -> bool {
    let na = normalize_path(a).trim_start_matches("a/").trim_start_matches("b/").to_string();
    let nb = normalize_path(b).trim_start_matches("a/").trim_start_matches("b/").to_string();

    na == nb || na.ends_with(&nb) || nb.ends_with(&na)
}

pub fn build_diff_evidences(
    semantic_files: &[FileSemanticInfo],
    staged_diff: &str,
    main_scopes: &[String],
) -> Vec<DiffEvidence> {
    let file_diffs = split_diff_by_file(staged_diff);
    println!("---- diff file map ----");
    for (path, _) in &file_diffs {
        println!("diff file: {}", path);
    }
    println!("-----------------------");
    let mut candidates: Vec<(i32, DiffEvidence)> = Vec::new();
    println!("---- semantic files ----");
    for info in semantic_files {
        println!("---- semantic files ----");
        if !main_scopes.is_empty() && !main_scopes.contains(&info.scope) {
            continue;
        }

        if let Some((_, diff_text)) = file_diffs.iter().find(|(path, _)| {
            same_file_path(path, &info.path)
            }) {
            let excerpt = trim_diff_excerpt(diff_text, 20);

            if excerpt.trim().is_empty() {
                continue;
            }

            let evidence = DiffEvidence {
                scope: info.scope.clone(),
                file_path: info.path.clone(),
                kind: info.kind.clone(),
                action: info.action.clone(),
                excerpt,
            };

            candidates.push((file_priority(info), evidence));
        }
    }
    println!("------------------------");

    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.file_path.cmp(&b.1.file_path))
    });

    candidates
        .into_iter()
        .take(6)
        .map(|(_, evidence)| evidence)
        .collect()
}
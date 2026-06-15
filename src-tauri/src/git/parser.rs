use crate::git::models::ParsedGitFile;

pub fn parse_git_status_line(line: &str) -> ParsedGitFile {
    let raw = line.to_string();

    if line.starts_with("??") {
        let path = line.get(3..).unwrap_or("").to_string();
        return ParsedGitFile {
            raw,
            x: "?".to_string(),
            y: "?".to_string(),
            path,
            original_path: None,
            change_type: "untracked".to_string(),
            staged: false,
            unstaged: true,
        };
    }

    let x = line.chars().nth(0).unwrap_or(' ');
    let y = line.chars().nth(1).unwrap_or(' ');
    let body = line.get(3..).unwrap_or("").to_string();

    if (x == 'R' || y == 'R') && body.contains(" -> ") {
        let parts: Vec<&str> = body.splitn(2, " -> ").collect();
        let original_path = parts.get(0).unwrap_or(&"").to_string();
        let path = parts.get(1).unwrap_or(&"").to_string();

        return ParsedGitFile {
            raw,
            x: x.to_string(),
            y: y.to_string(),
            path,
            original_path: Some(original_path),
            change_type: "renamed".to_string(),
            staged: x != ' ',
            unstaged: y != ' ',
        };
    }

    let status_char = if x != ' ' { x } else { y };

    let change_type = match status_char {
        'M' => "modified",
        'A' => "added",
        'D' => "deleted",
        'R' => "renamed",
        'C' => "copied",
        'U' => "updated-but-unmerged",
        _ => "unknown",
    }
    .to_string();

    ParsedGitFile {
        raw,
        x: x.to_string(),
        y: y.to_string(),
        path: body,
        original_path: None,
        change_type,
        staged: x != ' ',
        unstaged: y != ' ',
    }
}
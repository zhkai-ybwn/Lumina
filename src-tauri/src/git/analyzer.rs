use std::collections::BTreeMap;

use crate::git::config::{AnalysisConfig, SemanticRules};
use crate::git::models::{
    DiffSignals, FileRole, GitAnalysisContext, GitAnalysisCounts, ParsedGitFile, ScopeSummary,
};
use crate::git::parser::parse_git_status_line;

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn contains_any(path: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| path.contains(kw))
}

pub fn classify_file_role(path: &str, config: &AnalysisConfig) -> FileRole {
    let normalized = normalize_path(path);

    if contains_any(&normalized, &config.generated_keywords) {
        return FileRole::Generated;
    }

    if contains_any(&normalized, &config.tooling_keywords) {
        return FileRole::Tooling;
    }

    if contains_any(&normalized, &config.primary_keywords) {
        return FileRole::Primary;
    }

    if contains_any(&normalized, &config.secondary_keywords) {
        return FileRole::Secondary;
    }

    if normalized.starts_with("libs/feature/")
        || normalized.contains("/src/app/pages/")
        || normalized.contains("/src/app/views/")
        || normalized.contains("/src/app/components/")
    {
        return FileRole::Primary;
    }

    if normalized.starts_with("apps/")
        || normalized.starts_with("libs/shared/")
        || normalized.contains("/routes/")
        || normalized.contains("/i18n/")
    {
        return FileRole::Secondary;
    }

    FileRole::Secondary
}

pub fn extract_scope(path: &str) -> String {
    let normalized = normalize_path(path).to_lowercase();
    let rules = SemanticRules::default();

    for rule in rules.scope_rules {
        for pattern in rule.patterns {
            if normalized.starts_with(pattern) {
                let rest = normalized.trim_start_matches(pattern);
                let parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();

                match rule.name {
                    "feature" | "shared" => {
                        if let Some(first) = parts.first() {
                            return format!("{}/{}", rule.name, first);
                        }
                        return rule.name.to_string();
                    }
                    "app" => {
                        let all_parts: Vec<&str> =
                            normalized.split('/').filter(|s| !s.is_empty()).collect();

                        if all_parts.len() >= 2 {
                            let app_name = all_parts[1];

                            if normalized.contains("/routes/") {
                                return format!("app/{}/routes", app_name);
                            }

                            if normalized.contains("/i18n/") || normalized.contains("/locales/") {
                                return format!("app/{}/i18n", app_name);
                            }

                            if let Some(idx) = all_parts
                                .iter()
                                .position(|p| *p == "pages" || *p == "views" || *p == "components")
                            {
                                if all_parts.len() > idx + 1 {
                                    return format!("app/{}/{}", app_name, all_parts[idx + 1]);
                                }
                            }

                            return format!("app/{}", app_name);
                        }

                        return "app".to_string();
                    }
                    "tools" => {
                        if let Some(first) = parts.first() {
                            return format!("tools/{}", first);
                        }
                        return "tools".to_string();
                    }
                    "scripts" => {
                        return "scripts".to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    if normalized.contains("/src/assets/i18n/") {
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let filename = parts.last().copied().unwrap_or("i18n");
        return format!("i18n/{}", filename);
    }

    "root".to_string()
}

pub fn detect_diff_signals(files: &[ParsedGitFile]) -> DiffSignals {
    let mut has_route_changes = false;
    let mut has_i18n_changes = false;
    let mut has_constants_changes = false;
    let mut has_config_changes = false;
    let mut has_new_files = false;
    let mut has_deleted_files = false;
    let mut has_feature_code_changes = false;
    let mut has_renamed_files = false;
    let mut has_untracked_files = false;

    let mut has_page_like_changes = false;
    let mut has_component_like_changes = false;
    let mut has_service_like_changes = false;
    let mut has_model_like_changes = false;
    let mut has_log_like_changes = false;
    let mut has_form_like_changes = false;
    let mut has_list_like_changes = false;
    let mut has_detail_like_changes = false;

    for file in files {
        let path = normalize_path(&file.path);
        let lower = path.to_lowercase();

        if path.contains("/routes/") || path.contains("/router/") {
            has_route_changes = true;
        }

        if path.contains("/i18n/")
            || path.contains("/locales/")
            || path.contains("/src/assets/i18n/")
            || path.starts_with("tools/i18n/")
        {
            has_i18n_changes = true;
        }

        if path.contains("/constants/")
            || path.ends_with("/constants.ts")
            || path.ends_with(".constants.ts")
        {
            has_constants_changes = true;
        }

        if path.contains("/config/")
            || path.contains("/configs/")
            || path.contains("/configuration/")
            || path.contains("/configurations/")
            || path.ends_with(".config.ts")
            || path.ends_with(".config.js")
        {
            has_config_changes = true;
        }

        if file.change_type == "added" || file.change_type == "untracked" {
            has_new_files = true;
        }

        if file.change_type == "deleted" {
            has_deleted_files = true;
        }

        if file.change_type == "renamed" {
            has_renamed_files = true;
        }

        if file.change_type == "untracked" {
            has_untracked_files = true;
        }

        if path.contains("/feature/")
            || path.contains("/pages/")
            || path.contains("/views/")
            || path.contains("/components/")
            || path.contains("/services/")
            || path.contains("/modules/")
            || path.contains("/configuration/")
            || path.contains("/configurations/")
        {
            has_feature_code_changes = true;
        }

        if path.contains("/page/") || path.contains("/pages/") || path.contains(".page.") {
            has_page_like_changes = true;
        }

        if path.contains("/component/")
            || path.contains("/components/")
            || path.contains(".component.")
        {
            has_component_like_changes = true;
        }

        if path.contains("/service/")
            || path.contains("/services/")
            || path.contains(".service.")
        {
            has_service_like_changes = true;
        }

        if path.contains("/model/")
            || path.contains("/models/")
            || path.contains(".model.")
        {
            has_model_like_changes = true;
        }

        if lower.contains("log") {
            has_log_like_changes = true;
        }

        if lower.contains("form") || lower.contains("configuration") {
            has_form_like_changes = true;
        }

        if lower.contains("list") {
            has_list_like_changes = true;
        }

        if lower.contains("detail") || lower.contains("view") {
            has_detail_like_changes = true;
        }
    }

    DiffSignals {
        has_route_changes,
        has_i18n_changes,
        has_constants_changes,
        has_config_changes,
        has_new_files,
        has_deleted_files,
        has_feature_code_changes,
        has_renamed_files,
        has_untracked_files,
        has_page_like_changes,
        has_component_like_changes,
        has_service_like_changes,
        has_model_like_changes,
        has_log_like_changes,
        has_form_like_changes,
        has_list_like_changes,
        has_detail_like_changes,
    }
}

fn role_priority(role: &FileRole) -> i32 {
    match role {
        FileRole::Primary => 4,
        FileRole::Secondary => 3,
        FileRole::Tooling => 2,
        FileRole::Generated => 1,
    }
}

pub fn build_analysis_context(status_lines: &[String]) -> GitAnalysisContext {
    let config = AnalysisConfig::default();

    let parsed_files: Vec<ParsedGitFile> = status_lines
        .iter()
        .map(|line| parse_git_status_line(line))
        .collect();

    let mut primary_files = Vec::new();
    let mut secondary_files = Vec::new();
    let mut generated_files = Vec::new();
    let mut tooling_files = Vec::new();
    let mut untracked_files = Vec::new();
    let mut deleted_files = Vec::new();

    let mut scope_counter: BTreeMap<(String, FileRole), usize> = BTreeMap::new();

    for file in &parsed_files {
        let path = file.path.clone();
        let role = classify_file_role(&path, &config);
        let scope = extract_scope(&path);

        *scope_counter.entry((scope, role.clone())).or_insert(0) += 1;

        if file.change_type == "untracked" {
            untracked_files.push(path.clone());
        }

        if file.change_type == "deleted" {
            deleted_files.push(path.clone());
        }

        match role {
            FileRole::Primary => primary_files.push(path),
            FileRole::Secondary => secondary_files.push(path),
            FileRole::Generated => generated_files.push(path),
            FileRole::Tooling => tooling_files.push(path),
        }
    }

    let mut scope_summaries: Vec<ScopeSummary> = scope_counter
        .into_iter()
        .map(|((scope, role), count)| ScopeSummary { scope, role, count })
        .collect();

    scope_summaries.sort_by(|a, b| {
        role_priority(&b.role)
            .cmp(&role_priority(&a.role))
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.scope.cmp(&b.scope))
    });

    if scope_summaries.len() > config.max_scope_count {
        scope_summaries.truncate(config.max_scope_count);
    }

    let main_scopes: Vec<String> = scope_summaries.iter().map(|s| s.scope.clone()).collect();
    let diff_signals = detect_diff_signals(&parsed_files);

    let summary_hint = if !primary_files.is_empty() {
        "本次改动以业务功能代码调整为主，同时伴随部分配套路由、国际化、常量或配置更新。".to_string()
    } else if !secondary_files.is_empty() {
        "本次改动更偏向接入层、配置层或配套资源调整。".to_string()
    } else if !tooling_files.is_empty() {
        "本次改动主要集中在工具脚本或工程配置层面。".to_string()
    } else {
        "本次改动以零散文件更新为主。".to_string()
    };

    GitAnalysisContext {
        counts: GitAnalysisCounts {
            total: parsed_files.len(),
            primary: primary_files.len(),
            secondary: secondary_files.len(),
            generated: generated_files.len(),
            tooling: tooling_files.len(),
            untracked: untracked_files.len(),
            deleted: deleted_files.len(),
        },
        primary_files,
        secondary_files,
        generated_files,
        tooling_files,
        untracked_files,
        deleted_files,
        main_scopes,
        scope_summaries,
        diff_signals,
        summary_hint,
    }
}
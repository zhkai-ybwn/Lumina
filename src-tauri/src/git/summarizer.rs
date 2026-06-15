use std::collections::{BTreeMap, BTreeSet};

use crate::git::diff_parser::build_diff_evidences;
use crate::git::config::SemanticRules;
use crate::git::analyzer::{classify_file_role, extract_scope};
use crate::git::config::AnalysisConfig;
use crate::git::models::{
    CommitIntentSummary, FileAction, FileKind, FileRole, FileSemanticInfo, ParsedGitFile,
    ScopeActionSummary,
};
use crate::git::parser::parse_git_status_line;

fn detect_file_kind(path: &str) -> FileKind {
    let normalized = path.replace('\\', "/").to_lowercase();
    let rules = SemanticRules::default();

    for rule in rules.file_kind_rules {
        for pattern in rule.patterns {
            if normalized.contains(pattern) || normalized.ends_with(pattern) {
                return match rule.kind {
                    "page" => FileKind::Page,
                    "component" => FileKind::Component,
                    "service" => FileKind::Service,
                    "model" => FileKind::Model,
                    "config" => FileKind::Config,
                    "constants" => FileKind::Constants,
                    "route" => FileKind::Route,
                    "i18n" => FileKind::I18n,
                    "style" => FileKind::Style,
                    "test" => FileKind::Test,
                    _ => FileKind::Unknown,
                };
            }
        }
    }

    FileKind::Unknown
}

fn detect_file_action(file: &ParsedGitFile) -> FileAction {
    match file.change_type.as_str() {
        "added" | "untracked" => FileAction::Add,
        "deleted" => FileAction::Remove,
        "renamed" => FileAction::Rename,
        _ => FileAction::Update,
    }
}

fn action_label(kind: &FileKind, action: &FileAction) -> String {
    match (action, kind) {
        (FileAction::Add, FileKind::Page) => "add_pages".to_string(),
        (FileAction::Add, FileKind::Component) => "add_components".to_string(),
        (FileAction::Add, FileKind::Service) => "add_services".to_string(),
        (FileAction::Add, FileKind::Config) => "add_configuration".to_string(),
        (FileAction::Add, FileKind::Route) => "wire_routes".to_string(),
        (FileAction::Add, FileKind::I18n) => "add_translations".to_string(),

        (FileAction::Update, FileKind::Config) => "update_configuration".to_string(),
        (FileAction::Update, FileKind::Constants) => "update_constants".to_string(),
        (FileAction::Update, FileKind::Route) => "wire_routes".to_string(),
        (FileAction::Update, FileKind::I18n) => "update_translations".to_string(),
        (FileAction::Update, FileKind::Component) => "update_components".to_string(),
        (FileAction::Update, FileKind::Page) => "update_pages".to_string(),
        (FileAction::Update, FileKind::Service) => "update_services".to_string(),

        (FileAction::Remove, FileKind::Config) => "remove_obsolete_config".to_string(),
        (FileAction::Remove, FileKind::Component) => "remove_components".to_string(),
        (FileAction::Remove, FileKind::Route) => "remove_route_wiring".to_string(),

        (FileAction::Rename, _) => "rename_files".to_string(),

        _ => match action {
            FileAction::Add => "add_files".to_string(),
            FileAction::Update => "update_files".to_string(),
            FileAction::Remove => "remove_files".to_string(),
            FileAction::Rename => "rename_files".to_string(),
        },
    }
}

fn kind_label(kind: &FileKind) -> String {
    match kind {
        FileKind::Page => "page".to_string(),
        FileKind::Component => "component".to_string(),
        FileKind::Service => "service".to_string(),
        FileKind::Model => "model".to_string(),
        FileKind::Config => "config".to_string(),
        FileKind::Constants => "constants".to_string(),
        FileKind::Route => "route".to_string(),
        FileKind::I18n => "i18n".to_string(),
        FileKind::Style => "style".to_string(),
        FileKind::Test => "test".to_string(),
        FileKind::Unknown => "unknown".to_string(),
    }
}

fn score_file(info: &FileSemanticInfo) -> i32 {
    let mut score = 0;

    score += match info.role {
        FileRole::Primary => 3,
        FileRole::Secondary => 1,
        FileRole::Generated => -2,
        FileRole::Tooling => -1,
    };

    score += match info.kind {
        FileKind::Page | FileKind::Component | FileKind::Config => 4,
        FileKind::Service | FileKind::Model => 3,
        FileKind::Constants => 2,
        FileKind::Route | FileKind::I18n => 1,
        FileKind::Style | FileKind::Test => 0,
        FileKind::Unknown => 0,
    };

    score += match info.action {
        FileAction::Add => 3,
        FileAction::Update => 1,
        FileAction::Remove => 0,
        FileAction::Rename => 0,
    };

    score
}

pub fn build_file_semantics(status_lines: &[String]) -> Vec<FileSemanticInfo> {
    let config = AnalysisConfig::default();

    status_lines
        .iter()
        .map(|line| parse_git_status_line(line))
        .map(|file| {
            let role = classify_file_role(&file.path, &config);
            let scope = extract_scope(&file.path);
            if scope == "root" {
                println!("root scope file: {}", file.path);
            }
            let kind = detect_file_kind(&file.path);
            if kind == FileKind::Unknown {
                println!("unknown kind file: {}", file.path);
            }
            let action = detect_file_action(&file);

            FileSemanticInfo {
                path: file.path,
                scope,
                role,
                kind,
                action,
            }
        })
        .collect()
}

pub fn build_commit_intent_summary(
    status_lines: &[String],
    staged_diff: &str,
) -> CommitIntentSummary {
    let semantic_files = build_file_semantics(status_lines);

    let mut scope_scores: BTreeMap<String, i32> = BTreeMap::new();
    let mut scope_actions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut scope_kinds: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut scope_counts: BTreeMap<String, usize> = BTreeMap::new();

    let mut supporting_changes: BTreeSet<String> = BTreeSet::new();
    let mut risk_signals: BTreeSet<String> = BTreeSet::new();

    for info in &semantic_files {
        *scope_scores.entry(info.scope.clone()).or_insert(0) += score_file(info);
        *scope_counts.entry(info.scope.clone()).or_insert(0) += 1;

        scope_actions
            .entry(info.scope.clone())
            .or_default()
            .insert(action_label(&info.kind, &info.action));

        scope_kinds
            .entry(info.scope.clone())
            .or_default()
            .insert(kind_label(&info.kind));

        match info.kind {
            FileKind::Route => {
                supporting_changes.insert("routes".to_string());
                risk_signals.insert("route_wiring_changed".to_string());
            }
            FileKind::I18n => {
                supporting_changes.insert("i18n".to_string());
                risk_signals.insert("i18n_resources_changed".to_string());
            }
            FileKind::Constants => {
                risk_signals.insert("constants_changed".to_string());
            }
            FileKind::Config => {
                risk_signals.insert("configuration_flow_changed".to_string());
            }
            FileKind::Page => {
                risk_signals.insert("new_or_updated_pages".to_string());
            }
            _ => {}
        }

        if info.action == FileAction::Remove {
            risk_signals.insert("cleanup_or_removal_detected".to_string());
        }
    }

    let mut summaries: Vec<ScopeActionSummary> = scope_scores
        .iter()
        .map(|(scope, score)| ScopeActionSummary {
            scope: scope.clone(),
            score: *score,
            actions: scope_actions
                .get(scope)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default(),
            key_kinds: scope_kinds
                .get(scope)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default(),
            file_count: *scope_counts.get(scope).unwrap_or(&0),
        })
        .collect();

    summaries.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.file_count.cmp(&a.file_count))
            .then_with(|| a.scope.cmp(&b.scope))
    });

    let main_scopes = summaries
        .iter()
        .take(3)
        .map(|s| s.scope.clone())
        .collect::<Vec<_>>();

    let primary_actions = summaries.iter().take(3).cloned().collect::<Vec<_>>();
    let diff_evidences = build_diff_evidences(&semantic_files, staged_diff, &main_scopes);
    println!("---- commit intent summary ----");
    println!("main_scopes: {:#?}", main_scopes);
    println!("primary_actions: {:#?}", primary_actions);
    println!(
        "risk_signals: {:#?}",
        risk_signals.iter().cloned().collect::<Vec<_>>()
    );
    println!("diff_evidences: {:#?}", diff_evidences);
    println!("-------------------------------");
    CommitIntentSummary {
        main_scopes,
        primary_actions,
        supporting_changes: supporting_changes.into_iter().collect(),
        risk_signals: risk_signals.into_iter().collect(),
        diff_evidences,
    }
}
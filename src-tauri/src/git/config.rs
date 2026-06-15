#[derive(Debug, Clone)]
pub struct AiConfig {
    pub model: String,
    pub ollama_url: String,
    pub timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            model: "qwen3.5-9b-local".to_string(),
            ollama_url: "http://localhost:11434/api/generate".to_string(),
            timeout_secs: 180,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub primary_keywords: Vec<&'static str>,
    pub secondary_keywords: Vec<&'static str>,
    pub generated_keywords: Vec<&'static str>,
    pub tooling_keywords: Vec<&'static str>,
    pub max_preview_items: usize,
    pub max_diff_length: usize,
    pub max_scope_count: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            primary_keywords: vec![
                "/feature/",
                "/pages/",
                "/page/",
                "/views/",
                "/view/",
                "/components/",
                "/containers/",
                "/modules/",
                "/services/",
                "/store/",
                "/domain/",
                "/configuration/",
                "/configurations/",
            ],
            secondary_keywords: vec![
                "/routes/",
                "/router/",
                "/i18n/",
                "/locales/",
                "/constants/",
                "/config/",
                "/configs/",
                "/assets/",
                "/theme/",
                "/styles/",
            ],
            generated_keywords: vec![
                "/dist/",
                "/build/",
                "/coverage/",
                "/target/",
                "/node_modules/",
                "/src/assets/i18n/all/",
                ".map",
                "package-lock.json",
                "pnpm-lock.yaml",
                "yarn.lock",
            ],
            tooling_keywords: vec![
                "/scripts/",
                "/tools/",
                ".eslintrc",
                ".prettierrc",
                "tsconfig",
                "vite.config",
                "webpack.config",
                "jest.config",
                "vitest.config",
            ],
            max_preview_items: 20,
            max_diff_length: 10000,
            max_scope_count: 8,
        }
    }
}
#[derive(Debug, Clone)]
pub struct FileKindRule {
    /// 对应 models.rs 里的 FileKind 名称（小写字符串）
    pub kind: &'static str,
    /// 匹配模式（简单 contains/ends_with 规则）
    pub patterns: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ScopeRule {
    pub name: &'static str,
    pub patterns: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct SemanticRules {
    pub file_kind_rules: Vec<FileKindRule>,
    pub scope_rules: Vec<ScopeRule>,
}

impl Default for SemanticRules {
    fn default() -> Self {
        Self {
            file_kind_rules: vec![
                FileKindRule {
                    kind: "route",
                    patterns: vec!["/routes/", "/router/"],
                },
                FileKindRule {
                    kind: "component",
                    patterns: vec![".component.", "/components/"],
                },
                FileKindRule {
                    kind: "service",
                    patterns: vec![".service.", "/services/"],
                },
                FileKindRule {
                    kind: "config",
                    patterns: vec!["/configuration/", "/configurations/", ".config."],
                },
                FileKindRule {
                    kind: "constants",
                    patterns: vec!["/constants/", ".constants."],
                },
                FileKindRule {
                    kind: "i18n",
                    patterns: vec!["/i18n/", "/locales/", "/src/assets/i18n/"],
                },
                FileKindRule {
                    kind: "page",
                    patterns: vec!["/pages/", ".page."],
                },
                FileKindRule {
                    kind: "model",
                    patterns: vec!["/models/", ".model."],
                },
                FileKindRule {
                    kind: "test",
                    patterns: vec![".spec.", ".test."],
                },
            ],
            scope_rules: vec![
                ScopeRule {
                    name: "feature",
                    patterns: vec!["libs/feature/src/lib/"],
                },
                ScopeRule {
                    name: "shared",
                    patterns: vec!["libs/shared/src/lib/"],
                },
                ScopeRule {
                    name: "app",
                    patterns: vec!["apps/"],
                },
                ScopeRule {
                    name: "tools",
                    patterns: vec!["tools/"],
                },
                ScopeRule {
                    name: "scripts",
                    patterns: vec!["scripts/"],
                },
            ],
        }
    }
}
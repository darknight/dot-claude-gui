use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level Settings struct
// ---------------------------------------------------------------------------

/// Models Claude Code's `settings.json`. All fields are optional for forward
/// compatibility; unknown fields are preserved via the `extra` catch-all so
/// that a round-trip serialize → deserialize never loses data.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_co_authored_by: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,

    /// Maps event names (e.g. "PreToolUse") to arrays of HookGroups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HashMap<String, Vec<HookGroup>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_mcp_servers: Option<Vec<McpServerRef>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_line: Option<StatusLine>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_plugins: Option<HashMap<String, bool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_known_marketplaces: Option<HashMap<String, MarketplaceSource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_thinking_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_updates_channel: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_dangerous_mode_permission_prompt: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_overrides: Option<ModelOverrides>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_level: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode_per_session_opt_in: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_models: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_compact_window: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_clear_context_on_plan_accept: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_suggestion_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_memory_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_git_instructions: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub respect_gitignore: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_period_days: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_md_excludes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plans_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_highlighting_disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mcp_servers: Option<Vec<McpServerRef>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_mcpjson_servers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_mcpjson_servers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_all_project_mcp_servers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_mcp_servers_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_known_marketplaces: Option<Vec<MarketplaceSource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_marketplaces: Option<Vec<MarketplaceSource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_marketplaces: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_plugins: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_trust_message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_overrides: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_configs: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_all_hooks: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_http_hook_urls: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_hook_allowed_env_vars: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_hooks_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_permission_rules_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_skill_shell_execution: Option<bool>,

    // ---- M8 Advanced (Tier 3 scalar long tail) ----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_credential_export: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_auth_refresh: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_auth_refresh: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_method: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_org_uuid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_headers_helper: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_announcements: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_survey_rate: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_title_from_rename: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away_summary_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_thinking_summaries: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisor_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_dream_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_memory_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_listing_budget_fraction: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_listing_max_desc_chars: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_web_fetch_preflight: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_remote_settings_refresh: Option<bool>,

    // ---- M8 Sub-objects stored as raw JSON Value ----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_mode: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_suggestion: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_status_line: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spinner_verbs: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spinner_tips_override: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<serde_json::Value>,

    /// Preserves any fields not explicitly modelled above.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ask: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<HookDefinition>,

    /// Optional condition expression (sometimes called "if" in YAML-like configs).
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub hook_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_env_vars: Option<Vec<String>>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// MCP Server reference
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpServerRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// StatusLine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub status_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Marketplace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<MarketplaceSourceInfo>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSourceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// SandboxConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_read: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny_read: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow_write: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_commands: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_if_unavailable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_weaker_network_isolation: Option<bool>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// ModelOverrides
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelOverrides {
    #[serde(flatten)]
    pub overrides: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// EffectiveValue / ConfigSource
// ---------------------------------------------------------------------------

/// Wraps a value together with the configuration layer it originated from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectiveValue<T> {
    pub value: T,
    pub source: ConfigSource,
}

/// The configuration layer from which a value was resolved.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfigSource {
    Managed,
    User,
    Project,
    Local,
    Default,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
    }

    #[test]
    fn parse_minimal_settings() {
        let json = fixture("settings-minimal.json");
        let settings: Settings = serde_json::from_str(&json).expect("should parse minimal settings");
        // Minimal fixture has no explicit fields — everything is None / empty.
        assert!(settings.permissions.is_none());
        assert!(settings.hooks.is_none());
        assert!(settings.extra.is_empty());
    }

    #[test]
    fn parse_full_settings() {
        let json = fixture("settings-full.json");
        let settings: Settings = serde_json::from_str(&json).expect("should parse full settings");

        // Verify a handful of fields that the full fixture populates.
        let perms = settings.permissions.as_ref().expect("permissions should be present");
        assert!(!perms.allow.is_empty(), "allow list should be populated");

        assert!(settings.hooks.is_some(), "hooks should be present");
        assert!(settings.language.is_some(), "language should be present");
        assert!(settings.sandbox.is_some(), "sandbox should be present");
    }

    #[test]
    fn roundtrip_preserves_unknown_fields() {
        let json = fixture("settings-unknown-fields.json");
        let settings: Settings = serde_json::from_str(&json)
            .expect("should parse settings with unknown fields");

        // The fixture contains an unknown top-level key.
        assert!(
            !settings.extra.is_empty(),
            "extra map should capture unknown fields"
        );

        // Re-serialize and compare JSON objects (order-independent).
        let reserialized = serde_json::to_string(&settings).expect("serialize should succeed");
        let original_val: serde_json::Value =
            serde_json::from_str(&json).expect("original should be valid JSON");
        let roundtrip_val: serde_json::Value =
            serde_json::from_str(&reserialized).expect("reserialized should be valid JSON");

        assert_eq!(
            original_val, roundtrip_val,
            "round-trip should preserve all fields including unknown ones"
        );
    }

    #[test]
    fn empty_json_parses_to_default() {
        let settings: Settings = serde_json::from_str("{}").expect("empty object should parse");
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn settings_struct_matches_schema_snapshot() {
        // 读 docs/claude-schema-snapshot.json 的 settingsFields，
        // 断言每个字段在 Settings 里有同名字段或在跳过列表里。
        let snapshot_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/claude-schema-snapshot.json");
        let snapshot_raw = std::fs::read_to_string(&snapshot_path)
            .expect("schema snapshot should exist — run `pnpm extract:schema`");
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_raw).expect("snapshot should be valid JSON");

        let fields = snapshot["settingsFields"]
            .as_array()
            .expect("settingsFields should be an array")
            .iter()
            .map(|f| f["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();

        // 当前（M1 结束时）已建模的顶层字段：
        let modeled: &[&str] = &[
            "env",
            "includeCoAuthoredBy",
            "permissions",
            "hooks",
            "deniedMcpServers",
            "statusLine",
            "enabledPlugins",
            "extraKnownMarketplaces",
            "language",
            "alwaysThinkingEnabled",
            "autoUpdatesChannel",
            "minimumVersion",
            "skipDangerousModePermissionPrompt",
            "sandbox",
            "modelOverrides",
            "tui",
            "effortLevel",
            "model",
            "outputStyle",
            "fastMode",
            "fastModePerSessionOptIn",
            "availableModels",
            "autoCompactWindow",
            "showClearContextOnPlanAccept",
            "promptSuggestionEnabled",
            "autoMemoryEnabled",
            "includeGitInstructions",
            "respectGitignore",
            "cleanupPeriodDays",
            "claudeMdExcludes",
            "plansDirectory",
            "syntaxHighlightingDisabled",
            // M5: MCP tab
            "allowedMcpServers",
            "enabledMcpjsonServers",
            "disabledMcpjsonServers",
            "enableAllProjectMcpServers",
            "allowManagedMcpServersOnly",
            // M6: Plugins & Marketplace
            "strictKnownMarketplaces",
            "blockedMarketplaces",
            "skippedMarketplaces",
            "skippedPlugins",
            "pluginConfigs",
            "pluginTrustMessage",
            "skillOverrides",
            // M7: Hooks Policy
            "disableAllHooks",
            "allowedHttpHookUrls",
            "httpHookAllowedEnvVars",
            "allowManagedHooksOnly",
            "allowManagedPermissionRulesOnly",
            "disableSkillShellExecution",
            // M8: Advanced (Tier 3 long tail)
            "apiKeyHelper",
            "awsCredentialExport",
            "awsAuthRefresh",
            "gcpAuthRefresh",
            "forceLoginMethod",
            "forceLoginOrgUUID",
            "otelHeadersHelper",
            "prefersReducedMotion",
            "companyAnnouncements",
            "feedbackSurveyRate",
            "terminalTitleFromRename",
            "awaySummaryEnabled",
            "showThinkingSummaries",
            "advisorModel",
            "agent",
            "autoDreamEnabled",
            "autoMemoryDirectory",
            "skillListingBudgetFraction",
            "skillListingMaxDescChars",
            "skipWebFetchPreflight",
            "forceRemoteSettingsRefresh",
            // M8: sub-objects stored as serde_json::Value
            "attribution",
            "autoMode",
            "fileSuggestion",
            "worktree",
            "subagentStatusLine",
            "spinnerVerbs",
            "spinnerTipsOverride",
            "remote",
        ];

        // 在后续里程碑中添加字段时，从 `skipped` 列表移除并加到 `modeled`。
        let skipped: &[&str] = &[
            "$schema",
            // Defaults / additions that may appear and are not actionable in this plan:
            "disableBypassPermissionsMode", "disableDeepLinkRegistration",
            "additionalDirectories", "symlinkDirectories", "channelsEnabled",
            "allowedChannelPlugins", "voice",
            // Extra long-tail picked up by snapshot extraction (M8 will handle):
            "schema", "defaultShell", "disableAutoMode", "proxyAuthHelper",
            "spinnerTipsEnabled", "sshConfigs", "viewMode",
        ];

        let missing: Vec<&String> = fields
            .iter()
            .filter(|f| !modeled.contains(&f.as_str()) && !skipped.contains(&f.as_str()))
            .collect();

        assert!(
            missing.is_empty(),
            "Settings struct missing fields from snapshot: {:?}",
            missing
        );
    }

    #[test]
    fn parse_tui_field_is_typed() {
        let s: Settings =
            serde_json::from_str(r#"{"tui":"fullscreen"}"#).expect("should parse");
        assert_eq!(s.tui.as_deref(), Some("fullscreen"));
        assert!(!s.extra.contains_key("tui"));
    }

    #[test]
    fn parse_effort_level_is_typed() {
        let s: Settings =
            serde_json::from_str(r#"{"effortLevel":"xhigh"}"#).expect("should parse");
        assert_eq!(s.effort_level.as_deref(), Some("xhigh"));
        assert!(!s.extra.contains_key("effortLevel"));
    }

    #[test]
    fn parse_runtime_fields_are_typed() {
        let json = r#"{
            "model": "opus",
            "outputStyle": "default",
            "fastMode": true,
            "fastModePerSessionOptIn": false,
            "availableModels": ["sonnet", "opus"],
            "autoCompactWindow": 200000,
            "showClearContextOnPlanAccept": true,
            "promptSuggestionEnabled": false
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.model.as_deref(), Some("opus"));
        assert_eq!(s.output_style.as_deref(), Some("default"));
        assert_eq!(s.fast_mode, Some(true));
        assert_eq!(s.fast_mode_per_session_opt_in, Some(false));
        assert_eq!(
            s.available_models.as_ref().map(Vec::as_slice),
            Some(&["sonnet".to_string(), "opus".to_string()][..])
        );
        assert_eq!(s.auto_compact_window, Some(200000));
        assert_eq!(s.show_clear_context_on_plan_accept, Some(true));
        assert_eq!(s.prompt_suggestion_enabled, Some(false));
        for k in [
            "model",
            "outputStyle",
            "fastMode",
            "fastModePerSessionOptIn",
            "availableModels",
            "autoCompactWindow",
            "showClearContextOnPlanAccept",
            "promptSuggestionEnabled",
        ] {
            assert!(!s.extra.contains_key(k), "{} should be typed, not in extra", k);
        }
    }

    #[test]
    fn parse_general_extension_fields_are_typed() {
        let json = r#"{
            "autoMemoryEnabled": true,
            "includeGitInstructions": false,
            "respectGitignore": true,
            "cleanupPeriodDays": 60,
            "claudeMdExcludes": ["vendor/", ".venv/"],
            "plansDirectory": "~/.claude/plans",
            "syntaxHighlightingDisabled": false
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.auto_memory_enabled, Some(true));
        assert_eq!(s.include_git_instructions, Some(false));
        assert_eq!(s.respect_gitignore, Some(true));
        assert_eq!(s.cleanup_period_days, Some(60));
        assert_eq!(
            s.claude_md_excludes.as_ref().map(Vec::as_slice),
            Some(&["vendor/".to_string(), ".venv/".to_string()][..])
        );
        assert_eq!(s.plans_directory.as_deref(), Some("~/.claude/plans"));
        assert_eq!(s.syntax_highlighting_disabled, Some(false));
        for k in [
            "autoMemoryEnabled", "includeGitInstructions", "respectGitignore",
            "cleanupPeriodDays", "claudeMdExcludes", "plansDirectory",
            "syntaxHighlightingDisabled",
        ] {
            assert!(!s.extra.contains_key(k));
        }
    }
}

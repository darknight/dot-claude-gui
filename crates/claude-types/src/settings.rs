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
    pub enabled_plugins: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_known_marketplaces: Option<Vec<MarketplaceSource>>,

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
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

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
}

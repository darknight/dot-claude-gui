use std::collections::HashMap;

use claude_types::settings::{ConfigSource, Permissions, Settings};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A labeled configuration layer combining a source identifier with its settings.
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    pub source: ConfigSource,
    pub settings: Settings,
}

/// The result of merging multiple config layers, along with per-field source tracking.
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub settings: Settings,
    /// Maps field names to the ConfigSource that last set them.
    pub field_sources: HashMap<String, ConfigSource>,
}

// ---------------------------------------------------------------------------
// Merge engine
// ---------------------------------------------------------------------------

/// Merges layers in priority order: later layers override earlier ones.
///
/// Expected ordering: `[managed, user, project, local]`
///
/// Merge rules:
/// - Scalar `Option<T>` fields: later `Some` wins; `None` does not override.
/// - `env`: key-by-key merge, later layer wins per key.
/// - `permissions.allow/deny/ask`: CONCATENATE across layers.
/// - `permissions.defaultMode`: later layer overrides.
/// - `hooks`: merge by event-type key; later layer replaces the entire event-type entry.
/// - `enabledPlugins`: concatenate (Vec append, later layer appended last).
/// - `extraKnownMarketplaces`: concatenate (Vec append).
/// - `extra` (unknown fields): key-by-key merge, later layer wins.
pub fn merge_layers(layers: &[ConfigLayer]) -> MergedConfig {
    let mut settings = Settings::default();
    let mut field_sources: HashMap<String, ConfigSource> = HashMap::new();

    for layer in layers {
        let s = &layer.settings;
        let src = layer.source.clone();

        // ---- env: key-by-key merge -----------------------------------------
        if let Some(env) = &s.env {
            let merged_env = settings.env.get_or_insert_with(HashMap::new);
            for (k, v) in env {
                merged_env.insert(k.clone(), v.clone());
                field_sources.insert(format!("env.{}", k), src.clone());
            }
            field_sources.insert("env".to_string(), src.clone());
        }

        // ---- include_co_authored_by: scalar option -------------------------
        if let Some(v) = s.include_co_authored_by {
            settings.include_co_authored_by = Some(v);
            field_sources.insert("includeCoAuthoredBy".to_string(), src.clone());
        }

        // ---- permissions ---------------------------------------------------
        if let Some(perms) = &s.permissions {
            let merged_perms = settings.permissions.get_or_insert_with(Permissions::default);

            // allow/deny/ask: concatenate
            if !perms.allow.is_empty() {
                merged_perms.allow.extend(perms.allow.iter().cloned());
                field_sources.insert("permissions.allow".to_string(), src.clone());
            }
            if !perms.deny.is_empty() {
                merged_perms.deny.extend(perms.deny.iter().cloned());
                field_sources.insert("permissions.deny".to_string(), src.clone());
            }
            if !perms.ask.is_empty() {
                merged_perms.ask.extend(perms.ask.iter().cloned());
                field_sources.insert("permissions.ask".to_string(), src.clone());
            }

            // defaultMode: scalar override
            if let Some(dm) = &perms.default_mode {
                merged_perms.default_mode = Some(dm.clone());
                field_sources.insert("permissions.defaultMode".to_string(), src.clone());
            }

            // extra inside permissions: key-by-key
            for (k, v) in &perms.extra {
                merged_perms.extra.insert(k.clone(), v.clone());
                field_sources.insert(format!("permissions.extra.{}", k), src.clone());
            }

            field_sources.insert("permissions".to_string(), src.clone());
        }

        // ---- hooks: merge by event-type key --------------------------------
        if let Some(hooks) = &s.hooks {
            let merged_hooks = settings.hooks.get_or_insert_with(HashMap::new);
            for (event_type, groups) in hooks {
                merged_hooks.insert(event_type.clone(), groups.clone());
                field_sources.insert(format!("hooks.{}", event_type), src.clone());
            }
            field_sources.insert("hooks".to_string(), src.clone());
        }

        // ---- deniedMcpServers: scalar option (replace entire list) ---------
        if let Some(v) = &s.denied_mcp_servers {
            settings.denied_mcp_servers = Some(v.clone());
            field_sources.insert("deniedMcpServers".to_string(), src.clone());
        }

        // ---- statusLine: scalar option -------------------------------------
        if let Some(v) = &s.status_line {
            settings.status_line = Some(v.clone());
            field_sources.insert("statusLine".to_string(), src.clone());
        }

        // ---- enabledPlugins: concatenate -----------------------------------
        if let Some(plugins) = &s.enabled_plugins {
            let merged = settings.enabled_plugins.get_or_insert_with(Vec::new);
            merged.extend(plugins.iter().cloned());
            field_sources.insert("enabledPlugins".to_string(), src.clone());
        }

        // ---- extraKnownMarketplaces: concatenate ---------------------------
        if let Some(marketplaces) = &s.extra_known_marketplaces {
            let merged = settings.extra_known_marketplaces.get_or_insert_with(Vec::new);
            merged.extend(marketplaces.iter().cloned());
            field_sources.insert("extraKnownMarketplaces".to_string(), src.clone());
        }

        // ---- language: scalar option ---------------------------------------
        if let Some(v) = &s.language {
            settings.language = Some(v.clone());
            field_sources.insert("language".to_string(), src.clone());
        }

        // ---- alwaysThinkingEnabled: scalar option --------------------------
        if let Some(v) = s.always_thinking_enabled {
            settings.always_thinking_enabled = Some(v);
            field_sources.insert("alwaysThinkingEnabled".to_string(), src.clone());
        }

        // ---- autoUpdatesChannel: scalar option -----------------------------
        if let Some(v) = &s.auto_updates_channel {
            settings.auto_updates_channel = Some(v.clone());
            field_sources.insert("autoUpdatesChannel".to_string(), src.clone());
        }

        // ---- minimumVersion: scalar option ---------------------------------
        if let Some(v) = &s.minimum_version {
            settings.minimum_version = Some(v.clone());
            field_sources.insert("minimumVersion".to_string(), src.clone());
        }

        // ---- skipDangerousModePermissionPrompt: scalar option --------------
        if let Some(v) = s.skip_dangerous_mode_permission_prompt {
            settings.skip_dangerous_mode_permission_prompt = Some(v);
            field_sources.insert("skipDangerousModePermissionPrompt".to_string(), src.clone());
        }

        // ---- sandbox: scalar option ----------------------------------------
        if let Some(v) = &s.sandbox {
            settings.sandbox = Some(v.clone());
            field_sources.insert("sandbox".to_string(), src.clone());
        }

        // ---- modelOverrides: scalar option ----------------------------------
        if let Some(v) = &s.model_overrides {
            settings.model_overrides = Some(v.clone());
            field_sources.insert("modelOverrides".to_string(), src.clone());
        }

        // ---- extra (unknown fields): key-by-key ----------------------------
        for (k, v) in &s.extra {
            settings.extra.insert(k.clone(), v.clone());
            field_sources.insert(format!("extra.{}", k), src.clone());
        }
    }

    MergedConfig { settings, field_sources }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use claude_types::settings::{HookDefinition, HookGroup, Permissions, Settings};

    fn make_layer(source: ConfigSource, settings: Settings) -> ConfigLayer {
        ConfigLayer { source, settings }
    }

    // 1. empty_layers_produce_default
    #[test]
    fn empty_layers_produce_default() {
        let result = merge_layers(&[]);
        assert_eq!(result.settings, Settings::default());
        assert!(result.field_sources.is_empty());
    }

    // 2. single_layer_passes_through
    #[test]
    fn single_layer_passes_through() {
        let settings = Settings {
            language: Some("en".to_string()),
            ..Default::default()
        };
        let layer = make_layer(ConfigSource::User, settings.clone());
        let result = merge_layers(&[layer]);
        assert_eq!(result.settings.language, Some("en".to_string()));
        assert_eq!(result.field_sources.get("language"), Some(&ConfigSource::User));
    }

    // 3. later_layer_overrides_scalar_field
    #[test]
    fn later_layer_overrides_scalar_field() {
        let user_layer = make_layer(
            ConfigSource::User,
            Settings {
                language: Some("en".to_string()),
                ..Default::default()
            },
        );
        let project_layer = make_layer(
            ConfigSource::Project,
            Settings {
                language: Some("fr".to_string()),
                ..Default::default()
            },
        );
        let result = merge_layers(&[user_layer, project_layer]);
        assert_eq!(result.settings.language, Some("fr".to_string()));
        assert_eq!(result.field_sources.get("language"), Some(&ConfigSource::Project));
    }

    // 4. none_field_does_not_override
    #[test]
    fn none_field_does_not_override() {
        let user_layer = make_layer(
            ConfigSource::User,
            Settings {
                language: Some("en".to_string()),
                ..Default::default()
            },
        );
        // project layer has language: None — should NOT override
        let project_layer = make_layer(
            ConfigSource::Project,
            Settings {
                language: None,
                always_thinking_enabled: Some(true),
                ..Default::default()
            },
        );
        let result = merge_layers(&[user_layer, project_layer]);
        assert_eq!(result.settings.language, Some("en".to_string()));
        // The source for language should still be User
        assert_eq!(result.field_sources.get("language"), Some(&ConfigSource::User));
        // alwaysThinkingEnabled was set by project
        assert_eq!(result.settings.always_thinking_enabled, Some(true));
        assert_eq!(
            result.field_sources.get("alwaysThinkingEnabled"),
            Some(&ConfigSource::Project)
        );
    }

    // 5. permissions_allow_lists_concatenate
    #[test]
    fn permissions_allow_lists_concatenate() {
        let user_layer = make_layer(
            ConfigSource::User,
            Settings {
                permissions: Some(Permissions {
                    allow: vec!["Bash".to_string()],
                    deny: vec!["Write".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        let project_layer = make_layer(
            ConfigSource::Project,
            Settings {
                permissions: Some(Permissions {
                    allow: vec!["Read".to_string()],
                    deny: vec!["Edit".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        let result = merge_layers(&[user_layer, project_layer]);
        let perms = result.settings.permissions.expect("permissions should be present");
        assert_eq!(perms.allow, vec!["Bash".to_string(), "Read".to_string()]);
        assert_eq!(perms.deny, vec!["Write".to_string(), "Edit".to_string()]);
    }

    // 6. hooks_replaced_per_event_type
    #[test]
    fn hooks_replaced_per_event_type() {
        let hook_a = HookGroup {
            matcher: Some("*".to_string()),
            hooks: vec![HookDefinition {
                command: Some("echo user".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let hook_b = HookGroup {
            matcher: Some("*".to_string()),
            hooks: vec![HookDefinition {
                command: Some("echo project".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let hook_c = HookGroup {
            matcher: None,
            hooks: vec![HookDefinition {
                command: Some("echo postuse".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut user_hooks = HashMap::new();
        user_hooks.insert("PreToolUse".to_string(), vec![hook_a]);

        let mut project_hooks = HashMap::new();
        // replaces PreToolUse entirely
        project_hooks.insert("PreToolUse".to_string(), vec![hook_b]);
        // adds a new event type
        project_hooks.insert("PostToolUse".to_string(), vec![hook_c]);

        let user_layer = make_layer(
            ConfigSource::User,
            Settings {
                hooks: Some(user_hooks),
                ..Default::default()
            },
        );
        let project_layer = make_layer(
            ConfigSource::Project,
            Settings {
                hooks: Some(project_hooks),
                ..Default::default()
            },
        );

        let result = merge_layers(&[user_layer, project_layer]);
        let hooks = result.settings.hooks.expect("hooks should be present");

        // PreToolUse should have the project version (single group with "echo project")
        let pre_tool_use = hooks.get("PreToolUse").expect("PreToolUse should be present");
        assert_eq!(pre_tool_use.len(), 1);
        assert_eq!(
            pre_tool_use[0].hooks[0].command,
            Some("echo project".to_string())
        );

        // PostToolUse should be present from project layer
        assert!(hooks.contains_key("PostToolUse"));

        // Sources should attribute both to project
        assert_eq!(
            result.field_sources.get("hooks.PreToolUse"),
            Some(&ConfigSource::Project)
        );
        assert_eq!(
            result.field_sources.get("hooks.PostToolUse"),
            Some(&ConfigSource::Project)
        );
    }

    // 7. enabled_plugins_merge_per_key
    #[test]
    fn enabled_plugins_merge_per_key() {
        let user_layer = make_layer(
            ConfigSource::User,
            Settings {
                enabled_plugins: Some(vec!["plugin-a".to_string()]),
                ..Default::default()
            },
        );
        let project_layer = make_layer(
            ConfigSource::Project,
            Settings {
                enabled_plugins: Some(vec!["plugin-b".to_string()]),
                ..Default::default()
            },
        );
        let result = merge_layers(&[user_layer, project_layer]);
        let plugins = result.settings.enabled_plugins.expect("enabledPlugins should be present");
        assert!(plugins.contains(&"plugin-a".to_string()));
        assert!(plugins.contains(&"plugin-b".to_string()));
        // The field_sources for enabledPlugins should be Project (last layer that set it)
        assert_eq!(
            result.field_sources.get("enabledPlugins"),
            Some(&ConfigSource::Project)
        );
    }

    // 8. unknown_fields_merge_across_layers
    #[test]
    fn unknown_fields_merge_across_layers() {
        let mut user_extra = HashMap::new();
        user_extra.insert(
            "myCustomField".to_string(),
            serde_json::Value::String("user-value".to_string()),
        );

        let mut project_extra = HashMap::new();
        project_extra.insert(
            "anotherField".to_string(),
            serde_json::Value::Bool(true),
        );

        let user_layer = make_layer(
            ConfigSource::User,
            Settings {
                extra: user_extra,
                ..Default::default()
            },
        );
        let project_layer = make_layer(
            ConfigSource::Project,
            Settings {
                extra: project_extra,
                ..Default::default()
            },
        );

        let result = merge_layers(&[user_layer, project_layer]);

        // Both unknown fields should be present
        assert_eq!(
            result.settings.extra.get("myCustomField"),
            Some(&serde_json::Value::String("user-value".to_string()))
        );
        assert_eq!(
            result.settings.extra.get("anotherField"),
            Some(&serde_json::Value::Bool(true))
        );

        // Sources tracked
        assert_eq!(
            result.field_sources.get("extra.myCustomField"),
            Some(&ConfigSource::User)
        );
        assert_eq!(
            result.field_sources.get("extra.anotherField"),
            Some(&ConfigSource::Project)
        );
    }
}

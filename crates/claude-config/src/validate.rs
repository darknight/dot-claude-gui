use claude_types::api::ValidationError;
use claude_types::settings::Settings;

/// Valid values for `permissions.defaultMode`.
const VALID_DEFAULT_MODES: &[&str] = &[
    "acceptEdits",
    "bypassPermissions",
    "default",
    "dontAsk",
    "plan",
    "auto",
];

/// Valid hook event names.
const VALID_HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "Notification",
    "Stop",
    "SubagentStop",
    "CwdChanged",
    "FileChanged",
    "ConfigChange",
    "StopFailure",
    "TaskCreated",
    "WorktreeCreate",
    "WorktreeRemove",
    "PostCompact",
    "Elicitation",
    "InstructionsLoaded",
];

/// Validates the given `Settings` and returns a list of `ValidationError`s.
/// Returns an empty `Vec` if the settings are valid.
pub fn validate_settings(settings: &Settings) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate permissions.defaultMode
    if let Some(permissions) = &settings.permissions {
        if let Some(default_mode) = &permissions.default_mode {
            if !VALID_DEFAULT_MODES.contains(&default_mode.as_str()) {
                errors.push(ValidationError {
                    field: "permissions.defaultMode".to_string(),
                    message: format!(
                        "invalid defaultMode '{}'; must be one of: {}",
                        default_mode,
                        VALID_DEFAULT_MODES.join(", ")
                    ),
                });
            }
        }
    }

    // Validate hooks
    if let Some(hooks) = &settings.hooks {
        for (event_name, hook_groups) in hooks {
            // Validate event name
            if !VALID_HOOK_EVENTS.contains(&event_name.as_str()) {
                errors.push(ValidationError {
                    field: format!("hooks.{}", event_name),
                    message: format!(
                        "unknown hook event '{}'; must be one of: {}",
                        event_name,
                        VALID_HOOK_EVENTS.join(", ")
                    ),
                });
            }

            // Validate each hook group's hook definitions
            for (group_idx, group) in hook_groups.iter().enumerate() {
                for (hook_idx, hook) in group.hooks.iter().enumerate() {
                    let field_prefix = format!(
                        "hooks.{}[{}].hooks[{}]",
                        event_name, group_idx, hook_idx
                    );

                    match hook.hook_type.as_deref() {
                        Some("command") => {
                            // command type requires non-empty `command` field
                            let command_empty = hook
                                .command
                                .as_deref()
                                .map(|c| c.is_empty())
                                .unwrap_or(true);
                            if command_empty {
                                errors.push(ValidationError {
                                    field: format!("{}.command", field_prefix),
                                    message: "command hook requires a non-empty 'command' field"
                                        .to_string(),
                                });
                            }
                        }
                        Some("http") => {
                            // http type requires non-empty `url` field
                            let url_empty = hook
                                .url
                                .as_deref()
                                .map(|u| u.is_empty())
                                .unwrap_or(true);
                            if url_empty {
                                errors.push(ValidationError {
                                    field: format!("{}.url", field_prefix),
                                    message: "http hook requires a non-empty 'url' field"
                                        .to_string(),
                                });
                            }
                        }
                        Some(other) => {
                            errors.push(ValidationError {
                                field: format!("{}.type", field_prefix),
                                message: format!(
                                    "invalid hook type '{}'; must be 'command' or 'http'",
                                    other
                                ),
                            });
                        }
                        None => {
                            // No type specified — treat as missing/invalid
                            errors.push(ValidationError {
                                field: format!("{}.type", field_prefix),
                                message: "hook is missing required 'type' field; must be 'command' or 'http'"
                                    .to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Validate statusLine.type
    if let Some(status_line) = &settings.status_line {
        if let Some(status_type) = &status_line.status_type {
            if status_type != "command" {
                errors.push(ValidationError {
                    field: "statusLine.type".to_string(),
                    message: format!(
                        "invalid statusLine type '{}'; must be 'command'",
                        status_type
                    ),
                });
            }
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use claude_types::settings::{HookDefinition, HookGroup, Permissions, Settings};

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
    }

    fn make_settings_with_hook(event: &str, hook_type: Option<&str>, command: Option<&str>, url: Option<&str>) -> Settings {
        use std::collections::HashMap;
        let hook = HookDefinition {
            hook_type: hook_type.map(String::from),
            command: command.map(String::from),
            url: url.map(String::from),
            ..Default::default()
        };
        let group = HookGroup {
            hooks: vec![hook],
            ..Default::default()
        };
        let mut hooks = HashMap::new();
        hooks.insert(event.to_string(), vec![group]);
        Settings {
            hooks: Some(hooks),
            ..Default::default()
        }
    }

    #[test]
    fn valid_settings_no_errors() {
        let json = fixture("settings-full.json");
        let settings: Settings = serde_json::from_str(&json).expect("should parse full settings");
        let errors = validate_settings(&settings);
        assert!(
            errors.is_empty(),
            "expected no validation errors, got: {:?}",
            errors
        );
    }

    #[test]
    fn invalid_default_mode() {
        let settings = Settings {
            permissions: Some(Permissions {
                default_mode: Some("invalid_mode".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let errors = validate_settings(&settings);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "permissions.defaultMode");
    }

    #[test]
    fn unknown_hook_event() {
        let settings = make_settings_with_hook("InvalidEvent", Some("command"), Some("echo hi"), None);
        let errors = validate_settings(&settings);
        assert!(
            errors.iter().any(|e| e.field == "hooks.InvalidEvent"),
            "expected error for unknown hook event, got: {:?}",
            errors
        );
    }

    #[test]
    fn command_hook_missing_command() {
        let settings = make_settings_with_hook("PreToolUse", Some("command"), None, None);
        let errors = validate_settings(&settings);
        assert!(
            errors.iter().any(|e| e.field.ends_with(".command")),
            "expected error for missing command field, got: {:?}",
            errors
        );
    }

    #[test]
    fn http_hook_missing_url() {
        let settings = make_settings_with_hook("PostToolUse", Some("http"), None, None);
        let errors = validate_settings(&settings);
        assert!(
            errors.iter().any(|e| e.field.ends_with(".url")),
            "expected error for missing url field, got: {:?}",
            errors
        );
    }

    #[test]
    fn invalid_hook_type() {
        let settings = make_settings_with_hook("Stop", Some("websocket"), None, None);
        let errors = validate_settings(&settings);
        assert!(
            errors.iter().any(|e| e.field.ends_with(".type")),
            "expected error for invalid hook type, got: {:?}",
            errors
        );
    }

    #[test]
    fn default_settings_valid() {
        let settings = Settings::default();
        let errors = validate_settings(&settings);
        assert!(
            errors.is_empty(),
            "expected no errors for default settings, got: {:?}",
            errors
        );
    }
}

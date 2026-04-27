# Schema Triage — snapshot extras vs schemastore

**Date:** 2026-04-27
**Snapshot version:** 2.1.112 (cli.js extraction baseline)
**Compared against:** https://json.schemastore.org/claude-code-settings.json (fetched 2026-04-27)

Our `docs/claude-schema-snapshot.json` has **30 settingsFields** that schemastore does not. This document classifies them.

## Category A — Real settings, schemastore missing (27)

These are user-facing settings with substantive describes from cli.js source. Should be PR'd upstream to schemastore.

| Field | Type | Notes |
|---|---|---|
| `advisorModel` | string | Advisor model for the server-side advisor tool |
| `agent` | string | Built-in or custom agent for the main thread |
| `allowedChannelPlugins` | array | Teams/Enterprise allowlist of channel plugins |
| `autoCompactWindow` | number | Auto-compact window size |
| `autoDreamEnabled` | boolean | Background memory consolidation toggle |
| `autoMemoryDirectory` | string | Custom directory path for auto-memory storage |
| `channelsEnabled` | boolean | Teams/Enterprise channel notifications opt-in |
| `defaultShell` | enum | Default shell for input-box ! commands |
| `disableAutoMode` | enum | Disable auto mode |
| `disableSkillShellExecution` | boolean | Disable inline shell exec in skills/slash commands |
| `forceRemoteSettingsRefresh` | boolean | Block startup until managed settings refreshed |
| `gcpAuthRefresh` | string | Command to refresh GCP auth (paired with `awsCredentialExport` which IS in schemastore) |
| `minimumVersion` | string | Prevent downgrades on stable channel |
| `promptSuggestionEnabled` | boolean | Toggle prompt suggestions |
| `proxyAuthHelper` | string | Shell command outputting Proxy-Authorization header (EAP) |
| `remote` | object | Remote session configuration |
| `showClearContextOnPlanAccept` | boolean | Plan-approval dialog "clear context" option |
| `showThinkingSummaries` | boolean | Show thinking summaries in transcript view |
| `skillListingBudgetFraction` | number | Context window fraction for skill listing |
| `skillListingMaxDescChars` | number | Per-skill description char cap |
| `skillOverrides` | record | Per-skill listing overrides |
| `skipDangerousModePermissionPrompt` | boolean | Whether user accepted bypass-permissions dialog |
| `sshConfigs` | array | SSH connection configs (managed settings) |
| `subagentStatusLine` | object | Custom per-subagent status line |
| `syntaxHighlightingDisabled` | boolean | Disable syntax highlighting in diffs |
| `terminalTitleFromRename` | boolean | Whether /rename updates terminal title |
| `tui` | enum | Terminal UI renderer (`fullscreen` / `inline`) |
| `viewMode` | enum | Default transcript view mode |

## Category B — Internal (@internal marker, keep in our snapshot, don't PR) (2)

These have explicit `@internal` markers in describe text. Modeled in our Rust struct because we ship a GUI and may need to surface them, but should not be PR'd to schemastore (Anthropic intentionally keeps them out of public schema).

| Field | Type | Notes |
|---|---|---|
| `awaySummaryEnabled` | boolean | `@internal` Session recap on return |
| `voice` | object | `@internal` Voice handsfree settings, gated by `feature(VOICE_HANDSFREE)` |

## Category C — Extractor false positive (1)

| Field | Type | Reason to drop |
|---|---|---|
| `schema` | literal | Same conceptual key as `$schema` (JSON Schema reference). Our cli.js extractor saw `schema:y.literal(...)` and dropped the `$` prefix. Schemastore handles this via the standard `$schema` slot. **Drop from snapshot.** |

## Action items

- [ ] Implement a `docs/schema-extras.json` to merge into the schemastore-fetched snapshot, holding the 28 non-false-positive fields (Category A + B). When schemastore catches up, manually move fields out of extras.
- [ ] File a single tracking issue listing the 27 Category A fields for upstream PR(s) to schemastore.
- [ ] Drop `schema` (Category C) from snapshot.

## Coverage summary after triage

| Source | Count |
|---|---|
| schemastore (live fetch) | 60 |
| `docs/schema-extras.json` (Category A + B) | 28 |
| **Effective settingsFields snapshot** | **88** (≈ original 86 + 5 schemastore-only − 1 false positive − 2 sub-overlap; needs verification at regen time) |
| globalFields (frozen, ex cli.js) | 20 |

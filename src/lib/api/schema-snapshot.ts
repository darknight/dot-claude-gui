// Loads the schema snapshot at build time via Vite's JSON import.
// The snapshot lives at docs/claude-schema-snapshot.json.

import snapshot from "../../../docs/claude-schema-snapshot.json";

export interface SchemaField {
  name: string;
  type: string;
  optional?: boolean;
  enumValues?: string[];
  describe?: string;
}

export interface SchemaSnapshot {
  claudeCodeVersion: string;
  extractedAt: string;
  settingsFields: SchemaField[];
  globalFields: { name: string; source: string; type: string; describe?: string }[];
  warnings: string[];
}

export const schemaSnapshot: SchemaSnapshot = snapshot as unknown as SchemaSnapshot;

export function categorizeField(name: string): "auth" | "experience" | "subObject" | "misc" {
  const auth = new Set([
    "apiKeyHelper", "awsCredentialExport", "awsAuthRefresh", "gcpAuthRefresh",
    "forceLoginMethod", "forceLoginOrgUUID", "otelHeadersHelper", "proxyAuthHelper",
  ]);
  const experience = new Set([
    "prefersReducedMotion", "companyAnnouncements", "feedbackSurveyRate",
    "terminalTitleFromRename", "awaySummaryEnabled", "showThinkingSummaries",
    "spinnerTipsEnabled",
  ]);
  const sub = new Set([
    "attribution", "autoMode", "fileSuggestion", "worktree",
    "subagentStatusLine", "spinnerVerbs", "spinnerTipsOverride", "remote",
  ]);
  if (auth.has(name)) return "auth";
  if (experience.has(name)) return "experience";
  if (sub.has(name)) return "subObject";
  return "misc";
}

// TypeScript interfaces matching the Rust claude-types crate.
// All camelCase field names mirror the #[serde(rename_all = "camelCase")] on Rust structs.

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export interface Settings {
  env?: Record<string, string>;
  includeCoAuthoredBy?: boolean;
  permissions?: Permissions;
  hooks?: Record<string, HookGroup[]>;
  deniedMcpServers?: McpServerRef[];
  statusLine?: StatusLine;
  enabledPlugins?: string[];
  extraKnownMarketplaces?: MarketplaceSource[];
  language?: string;
  alwaysThinkingEnabled?: boolean;
  autoUpdatesChannel?: string;
  minimumVersion?: string;
  skipDangerousModePermissionPrompt?: boolean;
  sandbox?: SandboxConfig;
  modelOverrides?: Record<string, unknown>;
  // Catch-all for unknown / future fields preserved by the daemon.
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

export interface Permissions {
  allow?: string[];
  deny?: string[];
  ask?: string[];
  defaultMode?: string;
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

export interface HookGroup {
  matcher?: string;
  hooks?: HookDefinition[];
  /** Optional condition expression. */
  if?: string;
  [key: string]: unknown;
}

export interface HookDefinition {
  type?: "command" | "http";
  command?: string;
  url?: string;
  method?: string;
  headers?: Record<string, string>;
  timeout?: number;
  allowedEnvVars?: string[];
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// MCP Server reference
// ---------------------------------------------------------------------------

export interface McpServerRef {
  serverUrl?: string;
  serverName?: string;
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// StatusLine
// ---------------------------------------------------------------------------

export interface StatusLine {
  type?: string;
  command?: string;
  padding?: number;
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// Marketplace
// ---------------------------------------------------------------------------

export interface MarketplaceSourceInfo {
  type?: string;
  url?: string;
  [key: string]: unknown;
}

export interface MarketplaceSource {
  source?: MarketplaceSourceInfo;
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// SandboxConfig
// ---------------------------------------------------------------------------

export interface SandboxConfig {
  allowRead?: string[];
  denyRead?: string[];
  allowWrite?: string[];
  excludedCommands?: string[];
  failIfUnavailable?: boolean;
  enableWeakerNetworkIsolation?: boolean;
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// Config source / effective config
// ---------------------------------------------------------------------------

export type ConfigSource = "managed" | "user" | "project" | "local" | "default";

export interface EffectiveConfig {
  settings: Settings;
  fieldSources: Record<string, ConfigSource>;
}

// ---------------------------------------------------------------------------
// REST API response types
// ---------------------------------------------------------------------------

export interface ConfigResponse {
  settings: Settings;
  lastModified?: string;
  version?: string;
}

export interface ProjectEntry {
  id: string;
  path: string;
  name: string;
  registeredAt?: string;
}

export interface HealthResponse {
  status: string;
  version: string;
  claudeCodeVersion?: string;
  uptimeSeconds?: number;
}

export interface ValidationError {
  field: string;
  message: string;
}

export interface ErrorResponse {
  error: string;
  code?: string;
  message?: string;
  details?: ValidationError[];
  validationErrors?: ValidationError[];
}

// ---------------------------------------------------------------------------
// WebSocket event types (server → client)
// Discriminated union on the "type" field, mirroring WsEvent in Rust.
// ---------------------------------------------------------------------------

export type WsEvent =
  | {
      type: "configChanged";
      settings: Settings;
      source?: string;
    }
  | {
      type: "validationError";
      errors: WsValidationError[];
    }
  | {
      type: "commandOutput";
      commandId: string;
      line: string;
      stream: "stdout" | "stderr";
    }
  | {
      type: "commandCompleted";
      commandId: string;
      exitCode: number;
    }
  | {
      type: "connected";
      daemonVersion: string;
    };

export interface WsValidationError {
  field: string;
  message: string;
}

// ---------------------------------------------------------------------------
// WebSocket message types (client → server)
// ---------------------------------------------------------------------------

export type WsClientMessage =
  | { type: "subscribe"; topics: string[] }
  | { type: "unsubscribe"; topics: string[] };

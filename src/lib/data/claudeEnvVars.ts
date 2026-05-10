// Common Claude Code environment variables surfaced as a `<datalist>` in the
// launcher. Authoritative full list lives in the Claude Code docs; expand here
// as new ones become commonly used.

export interface ClaudeEnvVar {
  name: string;
  description: string;
}

export const CLAUDE_ENV_VARS: ClaudeEnvVar[] = [
  { name: "ANTHROPIC_API_KEY", description: "Anthropic API key" },
  { name: "ANTHROPIC_AUTH_TOKEN", description: "Pre-shared OAuth/auth token" },
  { name: "ANTHROPIC_BASE_URL", description: "Override API base URL" },
  { name: "ANTHROPIC_MODEL", description: "Override the default model" },
  { name: "CLAUDE_CODE_USE_BEDROCK", description: "Route requests through AWS Bedrock" },
  { name: "CLAUDE_CODE_USE_VERTEX", description: "Route requests through Google Vertex" },
  { name: "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", description: "Disable non-essential traffic" },
  { name: "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS", description: "Disable experimental betas" },
  { name: "CLAUDE_CODE_MAX_OUTPUT_TOKENS", description: "Max output tokens" },
  { name: "CLAUDE_CODE_OAUTH_TOKEN", description: "OAuth token (override stored auth)" },
  { name: "DISABLE_TELEMETRY", description: "Disable usage telemetry" },
  { name: "DISABLE_AUTOUPDATER", description: "Disable the auto-updater" },
  { name: "DISABLE_COST_WARNINGS", description: "Suppress cost warnings" },
  { name: "DISABLE_ERROR_REPORTING", description: "Disable error reporting" },
  { name: "MAX_THINKING_TOKENS", description: "Cap on thinking tokens" },
  { name: "BASH_DEFAULT_TIMEOUT_MS", description: "Default Bash tool timeout (ms)" },
  { name: "BASH_MAX_TIMEOUT_MS", description: "Max Bash tool timeout (ms)" },
  { name: "MCP_TIMEOUT", description: "MCP request timeout (ms)" },
  { name: "HTTP_PROXY", description: "HTTP proxy URL" },
  { name: "HTTPS_PROXY", description: "HTTPS proxy URL" },
];

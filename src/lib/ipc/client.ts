// src/lib/ipc/client.ts
//
// IPC client that mirrors DaemonClient's method surface but routes calls
// through Tauri IPC via invoke() instead of HTTP.

import { invoke } from "@tauri-apps/api/core";
import type {
  AccountOverview,
  AccountStatus,
  AddMcpServerRequest,
  AppMigrationReport,
  AvailablePlugin,
  ClaudeCliStatus,
  ClaudeMdFile,
  ClaudeMdFileDetail,
  ConfigResponse,
  DiskAccount,
  HealthResponse,
  LaunchConfig,
  LaunchRequest,
  MarketplaceInfo,
  McpServerInfo,
  MemoryFile,
  MemoryFileDetail,
  MemoryProject,
  PluginInfo,
  ProjectClaudeMdResponse,
  ProjectEffectiveResponse,
  ProjectEntry,
  ProjectMemoryListResponse,
  ProjectSettingsResponse,
  Settings,
  SkillContentResponse,
  SkillInfo,
} from "$lib/api/types.js";

// ---------------------------------------------------------------------------
// IpcError
// ---------------------------------------------------------------------------

/**
 * Error thrown by IpcClient methods. The Rust backend returns errors as
 * human-readable strings in the format "kind: details". We split on the first
 * colon so callers can pattern-match on the kind if needed.
 */
export class IpcError extends Error {
  constructor(public readonly kind: string, message: string) {
    super(message);
    this.name = "IpcError";
  }

  toString(): string {
    return `${this.kind}: ${this.message}`;
  }
}

function parseError(e: unknown): IpcError {
  const msg = typeof e === "string" ? e : String(e);
  const colonIdx = msg.indexOf(":");
  if (colonIdx > 0) {
    return new IpcError(msg.slice(0, colonIdx).trim(), msg.slice(colonIdx + 1).trim());
  }
  return new IpcError("unknown", msg);
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw parseError(e);
  }
}

// ---------------------------------------------------------------------------
// IpcClient
// ---------------------------------------------------------------------------

export class IpcClient {
  // --- migration (1) ---

  async takeMigrationReport(): Promise<AppMigrationReport | null> {
    return call("take_migration_report");
  }

  // --- health (1) ---

  async health(): Promise<HealthResponse> {
    return call("health");
  }

  // --- claude CLI pre-flight (1) ---

  async checkClaudeCli(): Promise<ClaudeCliStatus> {
    return call("check_claude_cli");
  }

  // --- config (2) ---

  async getUserConfig(): Promise<ConfigResponse> {
    return call("get_user_config");
  }

  async updateUserConfig(settings: Partial<Settings>): Promise<ConfigResponse> {
    // Rust: update_user_config(req: UpdateConfigRequest)
    // UpdateConfigRequest has { settings: Settings, if_match: Option<String> }
    return call("update_user_config", { req: { settings } });
  }

  // --- gui projects (6) ---

  async listProjects(): Promise<ProjectEntry[]> {
    return call("gui_list_projects");
  }

  async addProject(path: string): Promise<ProjectEntry> {
    return call("add_project", { req: { path } });
  }

  async bindProject(path: string, account: string): Promise<void> {
    return call("bind_project", { req: { path, account } });
  }

  async unbindProject(path: string): Promise<void> {
    return call("unbind_project", { req: { path } });
  }

  async removeProject(path: string): Promise<void> {
    return call("remove_project", { req: { path } });
  }

  async updateProjectLaunch(path: string, launch: LaunchConfig): Promise<void> {
    return call("update_project_launch", { req: { path, launch } });
  }

  async updateProjectPath(oldPath: string, newPath: string): Promise<void> {
    await call("update_project_path", { req: { oldPath, newPath } });
  }

  // --- plugins (8) ---

  async listPlugins(): Promise<PluginInfo[]> {
    return call("list_plugins");
  }

  async togglePlugin(id: string, enabled: boolean): Promise<void> {
    // Rust: toggle_plugin(id: String, enabled: bool)
    return call("toggle_plugin", { id, enabled });
  }

  async installPlugin(name: string, marketplace: string): Promise<{ requestId: string }> {
    // Rust: install_plugin(name: String, marketplace: String) -> CommandRequest
    // CommandRequest { request_id } serializes to { requestId }
    return call("install_plugin", { name, marketplace });
  }

  async uninstallPlugin(
    id: string,
    opts?: { accountName?: string; cwd?: string; scope?: string },
  ): Promise<{ requestId: string }> {
    // Rust: uninstall_plugin(id, accountName?, cwd?, scope?) -> CommandRequest
    // `scope` becomes `--scope <user|project>` on the CLI; required for
    // project-scope uninstalls (CLI defaults to user and rejects mismatch).
    return call("uninstall_plugin", {
      id,
      accountName: opts?.accountName,
      cwd: opts?.cwd,
      scope: opts?.scope,
    });
  }

  async listMarketplaces(): Promise<MarketplaceInfo[]> {
    return call("list_marketplaces");
  }

  async getMarketplacePlugins(marketplaceId: string): Promise<AvailablePlugin[]> {
    // Rust: get_marketplace_plugins(marketplace_id: String)
    return call("get_marketplace_plugins", { marketplaceId });
  }

  async addMarketplace(repo: string): Promise<{ requestId: string }> {
    // Rust: add_marketplace(repo: String) -> CommandRequest
    return call("add_marketplace", { repo });
  }

  async removeMarketplace(id: string): Promise<{ requestId: string }> {
    // Rust: remove_marketplace(id: String) -> CommandRequest
    return call("remove_marketplace", { id });
  }

  // --- mcp (3) ---

  async listMcpServers(): Promise<McpServerInfo[]> {
    return call("list_mcp_servers");
  }

  async addMcpServer(req: AddMcpServerRequest): Promise<{ requestId: string }> {
    // Rust: add_mcp_server(req: AddMcpServerRequest) -> CommandRequest
    return call("add_mcp_server", { req });
  }

  async removeMcpServer(name: string, scope?: string): Promise<{ requestId: string }> {
    // Rust: remove_mcp_server(name: String, scope: Option<String>) -> CommandRequest
    return call("remove_mcp_server", { name, scope });
  }

  // --- skills (2) ---

  async listSkills(): Promise<SkillInfo[]> {
    return call("list_skills");
  }

  async getSkillContent(id: string): Promise<SkillContentResponse> {
    // Rust: get_skill_content(id: String)
    return call("get_skill_content", { id });
  }

  async deleteUserSkill(id: string): Promise<void> {
    // Rust: delete_user_skill(id: String)
    return call("delete_user_skill", { id });
  }

  // --- claudemd (4) ---

  async listClaudeMdFiles(): Promise<ClaudeMdFile[]> {
    return call("list_claudemd_files");
  }

  async getClaudeMdFile(id: string): Promise<ClaudeMdFileDetail> {
    // Rust: get_claudemd_file(id: String)
    return call("get_claudemd_file", { id });
  }

  async updateClaudeMdFile(id: string, content: string): Promise<void> {
    // Rust: update_claudemd_file(id: String, content: String)
    return call("update_claudemd_file", { id, content });
  }

  async deleteClaudeMdFile(id: string): Promise<void> {
    // Rust: delete_claudemd_file(id: String)
    return call("delete_claudemd_file", { id });
  }

  // --- memory (5) ---

  async listMemoryProjects(): Promise<MemoryProject[]> {
    return call("list_memory_projects");
  }

  async listMemoryFiles(projectId: string): Promise<MemoryFile[]> {
    // Rust: list_memory_files(project_id: String)
    return call("list_memory_files", { projectId });
  }

  async getMemoryFile(projectId: string, filename: string): Promise<MemoryFileDetail> {
    // Rust: get_memory_file(project_id: String, filename: String)
    return call("get_memory_file", { projectId, filename });
  }

  async updateMemoryFile(accountName: string, projectId: string, filename: string, content: string): Promise<void> {
    // accountName is captured by the caller at click-time so the backend
    // can resolve the write path without going through state.current_dir(),
    // which can race with set_active_account.
    return call("update_memory_file", { accountName, projectId, filename, content });
  }

  async deleteMemoryFile(accountName: string, projectId: string, filename: string): Promise<void> {
    return call("delete_memory_file", { accountName, projectId, filename });
  }

  // --- launcher (2) ---

  async launchClaude(req: LaunchRequest): Promise<{ status: string }> {
    // Rust: launch_claude(req: LaunchRequest) -> serde_json::Value
    // Value is { status: "launched", projectPath: "..." }
    return call("launch_claude", { req });
  }

  async getClaudeArgs(): Promise<string> {
    // Rust: get_claude_args() -> String  (raw `claude --help` stdout)
    return call("get_claude_args");
  }

  // --- accounts (4) ---

  async listAccounts(): Promise<DiskAccount[]> {
    return call("list_accounts");
  }

  async createAccount(name: string): Promise<DiskAccount> {
    return call("create_account", { name });
  }

  async deleteAccount(name: string): Promise<void> {
    return call("delete_account", { name });
  }

  async getAccountStatus(name: string): Promise<AccountStatus> {
    return call("get_account_status", { name });
  }

  // --- account session (2) ---

  async setActiveAccount(name: string): Promise<string> {
    return call("set_active_account", { name });
  }

  async accountOverview(name: string): Promise<AccountOverview> {
    return call("account_overview", { name });
  }

  // --- project facets (10) — Stage 3 ---

  async projectReadSettings(projectPath: string): Promise<ProjectSettingsResponse> {
    return call("project_read_settings", { projectPath });
  }

  async projectWriteSettings(projectPath: string, settings: Settings): Promise<void> {
    return call("project_write_settings", { request: { projectPath, settings } });
  }

  async projectReadClaudeMd(projectPath: string): Promise<ProjectClaudeMdResponse> {
    return call("project_read_claudemd", { projectPath });
  }

  async projectWriteClaudeMd(projectPath: string, content: string): Promise<void> {
    return call("project_write_claudemd", { request: { projectPath, content } });
  }

  async projectListMemory(projectPath: string): Promise<ProjectMemoryListResponse> {
    return call("project_list_memory", { projectPath });
  }

  async projectReadMemoryFile(projectPath: string, fileName: string): Promise<string> {
    return call("project_read_memory_file", { request: { projectPath, fileName } });
  }

  async projectWriteMemoryFile(projectPath: string, fileName: string, content: string): Promise<void> {
    return call("project_write_memory_file", { request: { projectPath, fileName, content } });
  }

  async projectDeleteMemoryFile(projectPath: string, fileName: string): Promise<void> {
    return call("project_delete_memory_file", { request: { projectPath, fileName } });
  }

  async projectListPlugins(projectPath: string): Promise<PluginInfo[]> {
    return call("project_list_plugins", { projectPath });
  }

  async projectReadEffective(projectPath: string): Promise<ProjectEffectiveResponse> {
    return call("project_read_effective", { projectPath });
  }

  async getConfigDir(): Promise<string> {
    return call("get_config_dir");
  }
}

export const ipcClient = new IpcClient();

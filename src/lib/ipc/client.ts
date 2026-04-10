// src/lib/ipc/client.ts
//
// IPC client that mirrors DaemonClient's method surface but routes calls
// through Tauri IPC via invoke() instead of HTTP.

import { invoke } from "@tauri-apps/api/core";
import type {
  AddMcpServerRequest,
  AvailablePlugin,
  ClaudeMdFile,
  ClaudeMdFileDetail,
  ConfigResponse,
  EffectiveConfig,
  HealthResponse,
  LaunchRequest,
  MarketplaceInfo,
  McpServerInfo,
  MemoryFile,
  MemoryFileDetail,
  MemoryProject,
  PluginInfo,
  ProjectEntry,
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
  // --- health (1) ---

  async health(): Promise<HealthResponse> {
    return call("health");
  }

  // --- config (5) ---

  async getUserConfig(): Promise<ConfigResponse> {
    return call("get_user_config");
  }

  async updateUserConfig(settings: Partial<Settings>): Promise<ConfigResponse> {
    // Rust: update_user_config(req: UpdateConfigRequest)
    // UpdateConfigRequest has { settings: Settings, if_match: Option<String> }
    return call("update_user_config", { req: { settings } });
  }

  async getProjectConfig(projectId: string): Promise<ConfigResponse> {
    // Rust: get_project_config(project_id: String)
    return call("get_project_config", { projectId });
  }

  async updateProjectConfig(projectId: string, settings: Partial<Settings>): Promise<ConfigResponse> {
    // Rust: update_project_config(project_id: String, req: UpdateConfigRequest)
    return call("update_project_config", { projectId, req: { settings } });
  }

  async getEffectiveConfig(projectId: string): Promise<EffectiveConfig> {
    // Rust: get_effective_config(project_id: String) -> EffectiveConfigResponse
    // EffectiveConfigResponse { settings, field_sources } serializes to { settings, fieldSources }
    // which matches the TS EffectiveConfig type { settings, fieldSources }.
    return call("get_effective_config", { projectId });
  }

  // --- projects (3) ---

  async listProjects(): Promise<ProjectEntry[]> {
    return call("list_projects");
  }

  async registerProject(path: string): Promise<ProjectEntry> {
    // Rust: register_project(req: RegisterProjectRequest)
    // RegisterProjectRequest has { path: String, name: String } — name is extracted from path
    return call("register_project", { req: { path, name: "" } });
  }

  async unregisterProject(id: string): Promise<void> {
    // Rust: unregister_project(id: String)
    return call("unregister_project", { id });
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

  async uninstallPlugin(id: string): Promise<{ requestId: string }> {
    // Rust: uninstall_plugin(id: String) -> CommandRequest
    return call("uninstall_plugin", { id });
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

  async updateMemoryFile(projectId: string, filename: string, content: string): Promise<void> {
    // Rust: update_memory_file(project_id: String, filename: String, content: String)
    return call("update_memory_file", { projectId, filename, content });
  }

  async deleteMemoryFile(projectId: string, filename: string): Promise<void> {
    // Rust: delete_memory_file(project_id: String, filename: String)
    return call("delete_memory_file", { projectId, filename });
  }

  // --- launcher (1) ---

  async launchClaude(req: LaunchRequest): Promise<{ status: string }> {
    // Rust: launch_claude(req: LaunchRequest) -> serde_json::Value
    // Value is { status: "launched", projectPath: "..." }
    return call("launch_claude", { req });
  }
}

export const ipcClient = new IpcClient();

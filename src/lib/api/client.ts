import type {
  AddMcpServerRequest,
  AvailablePlugin,
  ClaudeMdFile,
  ClaudeMdFileDetail,
  ConfigResponse,
  EffectiveConfig,
  ErrorResponse,
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
  ValidationError,
} from "./types.js";

// ---------------------------------------------------------------------------
// ApiError
// ---------------------------------------------------------------------------

export class ApiError extends Error {
  readonly status: number;
  readonly details?: ValidationError[];

  constructor(message: string, status: number, details?: ValidationError[]) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.details = details;
  }
}

// ---------------------------------------------------------------------------
// DaemonClient
// ---------------------------------------------------------------------------

export class DaemonClient {
  private readonly baseUrl: string;
  private readonly token: string;

  constructor(baseUrl: string, token: string) {
    // Strip trailing slash for consistent URL construction.
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.token = token;
  }

  // -------------------------------------------------------------------------
  // Internal helpers
  // -------------------------------------------------------------------------

  private async fetch<T>(path: string, init?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${this.token}`,
      ...(init?.headers as Record<string, string> | undefined),
    };

    const response = await globalThis.fetch(url, { ...init, headers });

    if (!response.ok) {
      let details: ValidationError[] | undefined;
      let message = `HTTP ${response.status}`;

      try {
        const body: ErrorResponse = await response.json();
        message =
          body.message ?? body.error ?? message;
        details =
          body.details ?? body.validationErrors;
      } catch {
        // Body is not JSON — keep the default message.
      }

      throw new ApiError(message, response.status, details);
    }

    // 204 No Content — return void cast as T.
    if (response.status === 204) {
      return undefined as unknown as T;
    }

    return response.json() as Promise<T>;
  }

  // -------------------------------------------------------------------------
  // Health (no auth required by daemon, but we still add the header harmlessly)
  // -------------------------------------------------------------------------

  async health(): Promise<HealthResponse> {
    const url = `${this.baseUrl}/api/v1/health`;
    const response = await globalThis.fetch(url);
    if (!response.ok) {
      throw new ApiError(`HTTP ${response.status}`, response.status);
    }
    return response.json() as Promise<HealthResponse>;
  }

  // -------------------------------------------------------------------------
  // Config endpoints
  // -------------------------------------------------------------------------

  getUserConfig(): Promise<ConfigResponse> {
    return this.fetch<ConfigResponse>("/api/v1/config/user");
  }

  getProjectConfig(projectId: string): Promise<ConfigResponse> {
    return this.fetch<ConfigResponse>(
      `/api/v1/config/project/${encodeURIComponent(projectId)}`
    );
  }

  getEffectiveConfig(projectId: string): Promise<EffectiveConfig> {
    return this.fetch<EffectiveConfig>(
      `/api/v1/config/effective/${encodeURIComponent(projectId)}`
    );
  }

  updateUserConfig(settings: Partial<Settings>): Promise<ConfigResponse> {
    return this.fetch<ConfigResponse>("/api/v1/config/user", {
      method: "PUT",
      body: JSON.stringify({ settings }),
    });
  }

  async updateProjectConfig(projectId: string, settings: Partial<Settings>): Promise<ConfigResponse> {
    return this.fetch(`/api/v1/config/project/${projectId}`, {
      method: "PUT",
      body: JSON.stringify({ settings }),
    });
  }

  // -------------------------------------------------------------------------
  // Project endpoints
  // -------------------------------------------------------------------------

  listProjects(): Promise<ProjectEntry[]> {
    return this.fetch<ProjectEntry[]>("/api/v1/projects");
  }

  registerProject(path: string): Promise<ProjectEntry> {
    return this.fetch<ProjectEntry>("/api/v1/projects", {
      method: "POST",
      body: JSON.stringify({ path }),
    });
  }

  unregisterProject(id: string): Promise<void> {
    return this.fetch<void>(`/api/v1/projects/${encodeURIComponent(id)}`, {
      method: "DELETE",
    });
  }

  // -------------------------------------------------------------------------
  // Plugin endpoints
  // -------------------------------------------------------------------------

  listPlugins(): Promise<PluginInfo[]> {
    return this.fetch<PluginInfo[]>("/api/v1/plugins");
  }

  async togglePlugin(id: string, enabled: boolean): Promise<void> {
    await this.fetch(`/api/v1/plugins/${encodeURIComponent(id)}/toggle`, {
      method: "POST",
      body: JSON.stringify({ enabled }),
    });
  }

  installPlugin(name: string, marketplace: string): Promise<{ requestId: string }> {
    return this.fetch<{ requestId: string }>("/api/v1/plugins/install", {
      method: "POST",
      body: JSON.stringify({ name, marketplace }),
    });
  }

  uninstallPlugin(id: string): Promise<{ requestId: string }> {
    return this.fetch<{ requestId: string }>(
      `/api/v1/plugins/${encodeURIComponent(id)}/uninstall`,
      { method: "POST" }
    );
  }

  // -------------------------------------------------------------------------
  // Marketplace endpoints
  // -------------------------------------------------------------------------

  listMarketplaces(): Promise<MarketplaceInfo[]> {
    return this.fetch<MarketplaceInfo[]>("/api/v1/marketplaces");
  }

  getMarketplacePlugins(marketplaceId: string): Promise<AvailablePlugin[]> {
    return this.fetch<AvailablePlugin[]>(
      `/api/v1/marketplaces/${encodeURIComponent(marketplaceId)}/plugins`
    );
  }

  addMarketplace(repo: string): Promise<{ requestId: string }> {
    return this.fetch<{ requestId: string }>("/api/v1/marketplaces", {
      method: "POST",
      body: JSON.stringify({ repo }),
    });
  }

  removeMarketplace(id: string): Promise<{ requestId: string }> {
    return this.fetch<{ requestId: string }>(
      `/api/v1/marketplaces/${encodeURIComponent(id)}`,
      { method: "DELETE" }
    );
  }

  // -------------------------------------------------------------------------
  // Skills endpoints
  // -------------------------------------------------------------------------

  listSkills(): Promise<SkillInfo[]> {
    return this.fetch<SkillInfo[]>("/api/v1/skills");
  }

  getSkillContent(id: string): Promise<SkillContentResponse> {
    return this.fetch<SkillContentResponse>(
      `/api/v1/skills/${encodeURIComponent(id)}/content`
    );
  }

  // -------------------------------------------------------------------------
  // CLAUDE.md endpoints
  // -------------------------------------------------------------------------

  listClaudeMdFiles(): Promise<ClaudeMdFile[]> {
    return this.fetch<ClaudeMdFile[]>("/api/v1/claudemd");
  }

  getClaudeMdFile(id: string): Promise<ClaudeMdFileDetail> {
    return this.fetch<ClaudeMdFileDetail>(
      `/api/v1/claudemd/${encodeURIComponent(id)}`
    );
  }

  async updateClaudeMdFile(id: string, content: string): Promise<void> {
    await this.fetch(
      `/api/v1/claudemd/${encodeURIComponent(id)}`,
      {
        method: "PUT",
        body: JSON.stringify({ content }),
      }
    );
  }

  async deleteClaudeMdFile(id: string): Promise<void> {
    await this.fetch(
      `/api/v1/claudemd/${encodeURIComponent(id)}`,
      { method: "DELETE" }
    );
  }

  // -------------------------------------------------------------------------
  // Memory endpoints
  // -------------------------------------------------------------------------

  listMemoryProjects(): Promise<MemoryProject[]> {
    return this.fetch<MemoryProject[]>("/api/v1/memory");
  }

  listMemoryFiles(projectId: string): Promise<MemoryFile[]> {
    return this.fetch<MemoryFile[]>(`/api/v1/memory/${encodeURIComponent(projectId)}`);
  }

  getMemoryFile(projectId: string, filename: string): Promise<MemoryFileDetail> {
    return this.fetch<MemoryFileDetail>(
      `/api/v1/memory/${encodeURIComponent(projectId)}/${encodeURIComponent(filename)}`
    );
  }

  async updateMemoryFile(projectId: string, filename: string, content: string): Promise<void> {
    await this.fetch(
      `/api/v1/memory/${encodeURIComponent(projectId)}/${encodeURIComponent(filename)}`,
      {
        method: "PUT",
        body: JSON.stringify({ content }),
      }
    );
  }

  async deleteMemoryFile(projectId: string, filename: string): Promise<void> {
    await this.fetch(
      `/api/v1/memory/${encodeURIComponent(projectId)}/${encodeURIComponent(filename)}`,
      { method: "DELETE" }
    );
  }

  // -------------------------------------------------------------------------
  // MCP Server endpoints
  // -------------------------------------------------------------------------

  async listMcpServers(): Promise<McpServerInfo[]> {
    return this.fetch("/api/v1/mcp/servers");
  }

  async addMcpServer(req: AddMcpServerRequest): Promise<{ requestId: string }> {
    return this.fetch("/api/v1/mcp/servers", {
      method: "POST",
      body: JSON.stringify(req),
    });
  }

  async removeMcpServer(name: string, scope?: string): Promise<{ requestId: string }> {
    const params = scope ? `?scope=${encodeURIComponent(scope)}` : "";
    return this.fetch(`/api/v1/mcp/servers/${encodeURIComponent(name)}${params}`, {
      method: "DELETE",
    });
  }

  // -------------------------------------------------------------------------
  // Launcher endpoints
  // -------------------------------------------------------------------------

  async launchClaude(req: LaunchRequest): Promise<{ status: string }> {
    return this.fetch("/api/v1/launch", {
      method: "POST",
      body: JSON.stringify(req),
    });
  }
}

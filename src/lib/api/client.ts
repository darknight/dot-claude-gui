import type {
  ConfigResponse,
  EffectiveConfig,
  ErrorResponse,
  HealthResponse,
  ProjectEntry,
  Settings,
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

  updateUserConfig(settings: Settings): Promise<ConfigResponse> {
    return this.fetch<ConfigResponse>("/api/v1/config/user", {
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
}

import type { GuiMode } from "$lib/api/types";

const STORAGE_KEY_V1 = "dot-claude-gui-mode-v1";
const STORAGE_KEY_V2 = "dot-claude-gui-mode-v2";

export type ProjectFacetKey =
  | "binding"
  | "launch"
  | "plugins"
  | "settings"
  | "memory"
  | "claudemd"
  | "effective";

export type AccountFacetKey =
  | "overview"
  | "settings"
  | "plugins"
  | "skills"
  | "claudemd"
  | "memory"
  | "mcp";

const VALID_PROJECT_FACETS: readonly ProjectFacetKey[] = [
  "binding", "launch", "plugins", "settings", "memory", "claudemd", "effective",
];

const VALID_ACCOUNT_FACETS: readonly AccountFacetKey[] = [
  "overview", "settings", "plugins", "skills", "claudemd", "memory", "mcp",
];

const DEFAULT_ACCOUNT_FACET: AccountFacetKey = "overview";
const DEFAULT_PROJECT_FACET: ProjectFacetKey = "binding";

export type AccountSubsectionKey = "settingsSection" | "pluginsSection";
export type ProjectSubsectionKey = "settingsSection";

interface PerAccountUi {
  facet: AccountFacetKey;
  settingsSection?: string;
  pluginsSection?: string;
}

interface PerProjectUi {
  facet: ProjectFacetKey;
  settingsSection?: string;
}

interface PersistedModeV2 {
  version: 2;
  mode: GuiMode;
  selectedAccount: string | null;
  selectedProject: string | null;
  accounts: Record<string, PerAccountUi>;
  projects: Record<string, PerProjectUi>;
}

function defaultPersisted(): PersistedModeV2 {
  return {
    version: 2,
    mode: "account",
    selectedAccount: null,
    selectedProject: null,
    accounts: {},
    projects: {},
  };
}

function loadPersisted(): PersistedModeV2 {
  // v2 first.
  try {
    const raw = localStorage.getItem(STORAGE_KEY_V2);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && parsed.version === 2) {
        return {
          version: 2,
          mode: parsed.mode === "project" ? "project" : "account",
          selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
          selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
          accounts: sanitizeAccounts(parsed.accounts),
          projects: sanitizeProjects(parsed.projects),
        };
      }
    }
  } catch {
    // fall through
  }

  // v1 migration: keep mode + selections; drop global selectedProjectFacet on purpose
  // (per-project memory is the whole point of v2).
  try {
    const raw = localStorage.getItem(STORAGE_KEY_V1);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === "object") {
        return {
          version: 2,
          mode: parsed.mode === "project" ? "project" : "account",
          selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
          selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
          accounts: {},
          projects: {},
        };
      }
    }
  } catch {
    // fall through
  }

  return defaultPersisted();
}

function sanitizeAccounts(input: unknown): Record<string, PerAccountUi> {
  if (!input || typeof input !== "object") return {};
  const out: Record<string, PerAccountUi> = {};
  for (const [k, v] of Object.entries(input as Record<string, unknown>)) {
    if (!v || typeof v !== "object") continue;
    const raw = v as Record<string, unknown>;
    const facet = typeof raw.facet === "string" && VALID_ACCOUNT_FACETS.includes(raw.facet as AccountFacetKey)
      ? (raw.facet as AccountFacetKey)
      : DEFAULT_ACCOUNT_FACET;
    out[k] = {
      facet,
      settingsSection: typeof raw.settingsSection === "string" ? raw.settingsSection : undefined,
      pluginsSection: typeof raw.pluginsSection === "string" ? raw.pluginsSection : undefined,
    };
  }
  return out;
}

function sanitizeProjects(input: unknown): Record<string, PerProjectUi> {
  if (!input || typeof input !== "object") return {};
  const out: Record<string, PerProjectUi> = {};
  for (const [k, v] of Object.entries(input as Record<string, unknown>)) {
    if (!v || typeof v !== "object") continue;
    const raw = v as Record<string, unknown>;
    const facet = typeof raw.facet === "string" && VALID_PROJECT_FACETS.includes(raw.facet as ProjectFacetKey)
      ? (raw.facet as ProjectFacetKey)
      : DEFAULT_PROJECT_FACET;
    out[k] = {
      facet,
      settingsSection: typeof raw.settingsSection === "string" ? raw.settingsSection : undefined,
    };
  }
  return out;
}

class ModeStore {
  private _persisted = loadPersisted();
  mode = $state<GuiMode>(this._persisted.mode);
  selectedAccount = $state<string | null>(this._persisted.selectedAccount);
  selectedProject = $state<string | null>(this._persisted.selectedProject);
  accounts = $state<Record<string, PerAccountUi>>(this._persisted.accounts);
  projects = $state<Record<string, PerProjectUi>>(this._persisted.projects);

  setMode(m: GuiMode): void {
    this.mode = m;
    this.persist();
  }

  setSelectedAccount(name: string | null): void {
    this.selectedAccount = name;
    this.persist();
  }

  setSelectedProject(path: string | null): void {
    this.selectedProject = path;
    this.persist();
  }

  // Account-side facet/subsection

  accountFacet(name: string | null): AccountFacetKey {
    if (!name) return DEFAULT_ACCOUNT_FACET;
    return this.accounts[name]?.facet ?? DEFAULT_ACCOUNT_FACET;
  }

  setAccountFacet(name: string, facet: AccountFacetKey): void {
    const prev = this.accounts[name] ?? { facet: DEFAULT_ACCOUNT_FACET };
    this.accounts = { ...this.accounts, [name]: { ...prev, facet } };
    this.persist();
  }

  accountSubsection(name: string | null, key: AccountSubsectionKey): string | undefined {
    if (!name) return undefined;
    return this.accounts[name]?.[key];
  }

  setAccountSubsection(name: string, key: AccountSubsectionKey, val: string): void {
    const prev = this.accounts[name] ?? { facet: DEFAULT_ACCOUNT_FACET };
    this.accounts = { ...this.accounts, [name]: { ...prev, [key]: val } };
    this.persist();
  }

  // Project-side facet/subsection

  projectFacet(path: string | null): ProjectFacetKey {
    if (!path) return DEFAULT_PROJECT_FACET;
    return this.projects[path]?.facet ?? DEFAULT_PROJECT_FACET;
  }

  setProjectFacet(path: string, facet: ProjectFacetKey): void {
    const prev = this.projects[path] ?? { facet: DEFAULT_PROJECT_FACET };
    this.projects = { ...this.projects, [path]: { ...prev, facet } };
    this.persist();
  }

  projectSubsection(path: string | null, key: ProjectSubsectionKey): string | undefined {
    if (!path) return undefined;
    return this.projects[path]?.[key];
  }

  setProjectSubsection(path: string, key: ProjectSubsectionKey, val: string): void {
    const prev = this.projects[path] ?? { facet: DEFAULT_PROJECT_FACET };
    this.projects = { ...this.projects, [path]: { ...prev, [key]: val } };
    this.persist();
  }

  // Cleanup

  pruneStale(validAccountNames: Set<string>, validProjectPaths: Set<string>): void {
    let mutated = false;
    const nextAccounts: Record<string, PerAccountUi> = {};
    for (const [k, v] of Object.entries(this.accounts)) {
      if (validAccountNames.has(k)) nextAccounts[k] = v;
      else mutated = true;
    }
    const nextProjects: Record<string, PerProjectUi> = {};
    for (const [k, v] of Object.entries(this.projects)) {
      if (validProjectPaths.has(k)) nextProjects[k] = v;
      else mutated = true;
    }
    if (mutated) {
      this.accounts = nextAccounts;
      this.projects = nextProjects;
      this.persist();
    }
  }

  private persist(): void {
    try {
      const snapshot: PersistedModeV2 = {
        version: 2,
        mode: this.mode,
        selectedAccount: this.selectedAccount,
        selectedProject: this.selectedProject,
        accounts: this.accounts,
        projects: this.projects,
      };
      localStorage.setItem(STORAGE_KEY_V2, JSON.stringify(snapshot));
    } catch {
      // localStorage unavailable — ignore
    }
  }
}

export const modeStore = new ModeStore();

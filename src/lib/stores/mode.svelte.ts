import type { GuiMode } from "$lib/api/types";

const STORAGE_KEY = "dot-claude-gui-mode-v1";

export type ProjectFacetKey =
  | "binding"
  | "launch"
  | "plugins"
  | "settings"
  | "memory"
  | "claudemd"
  | "effective";

const VALID_FACETS: readonly ProjectFacetKey[] = [
  "binding",
  "launch",
  "plugins",
  "settings",
  "memory",
  "claudemd",
  "effective",
];

interface PersistedMode {
  mode: GuiMode;
  selectedAccount: string | null;
  selectedProject: string | null;
  selectedProjectFacet: ProjectFacetKey;
}

function loadPersisted(): PersistedMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      const facet = VALID_FACETS.includes(parsed.selectedProjectFacet)
        ? (parsed.selectedProjectFacet as ProjectFacetKey)
        : "binding";
      return {
        mode: parsed.mode === "project" ? "project" : "account",
        selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
        selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
        selectedProjectFacet: facet,
      };
    }
  } catch {
    // fall through to defaults
  }
  return {
    mode: "account",
    selectedAccount: null,
    selectedProject: null,
    selectedProjectFacet: "binding",
  };
}

class ModeStore {
  private _persisted = loadPersisted();
  mode = $state<GuiMode>(this._persisted.mode);
  selectedAccount = $state<string | null>(this._persisted.selectedAccount);
  selectedProject = $state<string | null>(this._persisted.selectedProject);
  selectedProjectFacet = $state<ProjectFacetKey>(this._persisted.selectedProjectFacet);

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

  setSelectedProjectFacet(facet: ProjectFacetKey): void {
    this.selectedProjectFacet = facet;
    this.persist();
  }

  private persist(): void {
    try {
      const snapshot: PersistedMode = {
        mode: this.mode,
        selectedAccount: this.selectedAccount,
        selectedProject: this.selectedProject,
        selectedProjectFacet: this.selectedProjectFacet,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
    } catch {
      // localStorage unavailable — ignore
    }
  }
}

export const modeStore = new ModeStore();

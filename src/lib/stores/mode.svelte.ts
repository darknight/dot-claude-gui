import type { GuiMode } from "$lib/api/types";

const STORAGE_KEY = "dot-claude-gui-mode-v1";

interface PersistedMode {
  mode: GuiMode;
  selectedAccount: string | null;
  selectedProject: string | null;
}

function loadPersisted(): PersistedMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        mode: parsed.mode === "project" ? "project" : "account",
        selectedAccount: typeof parsed.selectedAccount === "string" ? parsed.selectedAccount : null,
        selectedProject: typeof parsed.selectedProject === "string" ? parsed.selectedProject : null,
      };
    }
  } catch {
    // fall through to defaults
  }
  return { mode: "account", selectedAccount: null, selectedProject: null };
}

class ModeStore {
  private _persisted = loadPersisted();
  mode = $state<GuiMode>(this._persisted.mode);
  selectedAccount = $state<string | null>(this._persisted.selectedAccount);
  selectedProject = $state<string | null>(this._persisted.selectedProject);

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

  private persist(): void {
    try {
      const snapshot: PersistedMode = {
        mode: this.mode,
        selectedAccount: this.selectedAccount,
        selectedProject: this.selectedProject,
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
    } catch {
      // localStorage unavailable — ignore
    }
  }
}

export const modeStore = new ModeStore();

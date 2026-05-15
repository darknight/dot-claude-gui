import { ipcClient } from "$lib/ipc/client.js";
import type { MemoryProject, MemoryFile, MemoryFileDetail } from "$lib/api/types";
import { toastStore } from "./toast.svelte";

class MemoryStore {
  projects = $state<MemoryProject[]>([]);
  activeProjectId = $state<string | null>(null);
  files = $state<MemoryFile[]>([]);
  activeFile = $state<MemoryFileDetail | null>(null);
  activeFileDirty = $state<boolean>(false);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  // Per-account caches so re-entering an account is instant:
  //  - rememberedProjectByAccount: last picked project, restored on switch
  //  - projectsByAccount: cached project list, lets the dropdown render
  //    immediately without waiting for an IPC roundtrip
  // Encoded project ids can collide across accounts (same path → same id),
  // so both MUST key on account name, not just id.
  // Session-scoped (not persisted across app restarts).
  private currentAccount: string | null = null;
  private rememberedProjectByAccount: Map<string, string> = new Map();
  private projectsByAccount: Map<string, MemoryProject[]> = new Map();

  hasProjectsCached(name: string): boolean {
    return this.projectsByAccount.has(name);
  }

  /** Switch the per-account memory context. Wipes interaction state and
   *  fills `projects` from the cached snapshot if any (so the dropdown
   *  renders instantly on re-entry). Call before any IPC; pair with
   *  `restoreSelection()` once `active_account_dir` has been switched on
   *  the backend. */
  switchAccount(name: string): void {
    this.currentAccount = name;
    this.projects = this.projectsByAccount.get(name) ?? [];
    this.activeProjectId = null;
    this.files = [];
    this.activeFile = null;
    this.activeFileDirty = false;
    this.loading = false;
    this.saving = false;
    this.error = "";
  }

  /** Reapply the remembered selection for the current account, if it still
   *  exists in `projects`. No-op when there's nothing remembered. */
  restoreSelection(): void {
    if (this.currentAccount === null) return;
    const remembered = this.rememberedProjectByAccount.get(this.currentAccount);
    if (remembered && this.projects.some((p) => p.id === remembered)) {
      this.selectProject(remembered);
    }
  }

  async loadProjects() {
    this.loading = true;
    this.error = "";
    try {
      const fresh = await ipcClient.listMemoryProjects();
      this.projects = fresh;
      if (this.currentAccount !== null) {
        this.projectsByAccount.set(this.currentAccount, fresh);
      }
      // Drop the prior selection if it isn't in the refreshed list (e.g. the
      // project's memory dir was deleted out from under us).
      if (
        this.activeProjectId !== null &&
        !fresh.some((p) => p.id === this.activeProjectId)
      ) {
        this.clearSelection();
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load memory projects";
    } finally {
      this.loading = false;
    }
  }

  async loadFiles(projectId: string) {
    this.loading = true;
    this.error = "";
    try {
      this.files = await ipcClient.listMemoryFiles(projectId);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load memory files";
    } finally {
      this.loading = false;
    }
  }

  async loadFile(projectId: string, filename: string) {
    this.loading = true;
    this.error = "";
    try {
      this.activeFile = await ipcClient.getMemoryFile(projectId, filename);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load memory file";
    } finally {
      this.loading = false;
    }
  }

  async saveFile(projectId: string, filename: string, content: string) {
    this.saving = true;
    this.error = "";
    try {
      await ipcClient.updateMemoryFile(projectId, filename, content);
      // Update activeFile content to reflect saved state
      if (this.activeFile && this.activeFile.filename === filename) {
        this.activeFile = { ...this.activeFile, content };
        toastStore.success("File saved");
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save memory file";
      toastStore.error(this.error);
    } finally {
      this.saving = false;
    }
  }

  async deleteFile(projectId: string, filename: string) {
    this.error = "";
    try {
      await ipcClient.deleteMemoryFile(projectId, filename);
      this.files = this.files.filter((f) => f.filename !== filename);
      if (this.activeFile?.filename === filename) {
        this.activeFile = null;
        toastStore.success("File deleted");
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to delete memory file";
      toastStore.error(this.error);
    }
  }

  selectProject(id: string) {
    if (this.currentAccount !== null) {
      this.rememberedProjectByAccount.set(this.currentAccount, id);
    }
    this.activeProjectId = id;
    this.files = [];
    this.activeFile = null;
    this.activeFileDirty = false;
    void this.loadFiles(id);
  }

  clearSelection() {
    if (this.currentAccount !== null) {
      this.rememberedProjectByAccount.delete(this.currentAccount);
    }
    this.activeProjectId = null;
    this.files = [];
    this.activeFile = null;
    this.activeFileDirty = false;
  }

  selectFile(filename: string) {
    if (this.activeProjectId) {
      void this.loadFile(this.activeProjectId, filename);
    }
  }

  reset(): void {
    this.projects = [];
    this.activeProjectId = null;
    this.files = [];
    this.activeFile = null;
    this.activeFileDirty = false;
    this.loading = false;
    this.saving = false;
    this.error = "";
  }
}

export const memoryStore = new MemoryStore();

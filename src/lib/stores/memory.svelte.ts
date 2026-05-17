import { ipcClient } from "$lib/ipc/client.js";
import type { MemoryProject, MemoryFile, MemoryFileDetail } from "$lib/api/types";
import { toastStore } from "./toast.svelte";

class MemoryStore {
  projects = $state<MemoryProject[]>([]);
  // Lazy-loaded file lists, keyed by project id. Populated on first expand
  // (toggleProject) and kept until switchAccount/reset.
  filesByProject = $state<Record<string, MemoryFile[]>>({});
  // UI collapse state per project. In-memory only, not persisted to disk.
  expanded = $state<Record<string, boolean>>({});
  // The project owning the file currently open in the editor. Tracked passively
  // as a side effect of selectFile so save/delete know where to write.
  activeProjectId = $state<string | null>(null);
  activeFile = $state<MemoryFileDetail | null>(null);
  activeFileDirty = $state<boolean>(false);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  // Per-account caches so re-entering an account is instant:
  //  - rememberedProjectByAccount: last project a file was opened from; used
  //    by restoreSelection() to auto-expand that group on re-entry
  //  - projectsByAccount: cached project list so the sidebar paints before
  //    the IPC roundtrip lands
  // Encoded project ids can collide across accounts (same path → same id),
  // so both MUST key on account name, not just id.
  // Session-scoped (not persisted across app restarts).
  private currentAccount: string | null = null;
  private rememberedProjectByAccount: Map<string, string> = new Map();
  private projectsByAccount: Map<string, MemoryProject[]> = new Map();
  // Monotonic counter to discard stale selectFile IPC responses. Bumped on
  // each call; in the await callback we re-check it before committing state,
  // so when the user clicks two files in quick succession only the latest
  // result wins (and activeProjectId/activeFile can't drift out of sync —
  // which is what would let save/delete write to the wrong project).
  private selectSeq = 0;

  hasProjectsCached(name: string): boolean {
    return this.projectsByAccount.has(name);
  }

  /** Switch the per-account memory context. Wipes interaction state and
   *  fills `projects` from the cached snapshot if any (so the list renders
   *  instantly on re-entry). Call before any IPC; pair with
   *  `restoreSelection()` once `active_account_dir` has been switched on
   *  the backend. */
  switchAccount(name: string): void {
    this.currentAccount = name;
    this.projects = this.projectsByAccount.get(name) ?? [];
    this.filesByProject = {};
    this.expanded = {};
    this.activeProjectId = null;
    this.activeFile = null;
    this.activeFileDirty = false;
    this.loading = false;
    this.saving = false;
    this.error = "";
    // Discard any in-flight selectFile from the prior account so its IPC
    // response can't commit a stale file/project under the new account.
    this.selectSeq++;
  }

  /** Auto-expand the project last interacted with for this account, so its
   *  files render without a manual click. No-op when nothing's remembered or
   *  the remembered project no longer exists. */
  async restoreSelection(): Promise<void> {
    if (this.currentAccount === null) return;
    const remembered = this.rememberedProjectByAccount.get(this.currentAccount);
    if (!remembered) return;
    if (!this.projects.some((p) => p.id === remembered)) return;
    this.expanded = { ...this.expanded, [remembered]: true };
    if (!this.filesByProject[remembered]) {
      await this.loadFilesFor(remembered);
    }
  }

  async loadProjects() {
    // Capture the account context at call-time. If switchAccount lands
    // before the IPC resolves, the snapshot won't match and we discard
    // the result — it belongs to a stale account.
    const accountAtStart = this.currentAccount;
    this.loading = true;
    this.error = "";
    try {
      const fresh = await ipcClient.listMemoryProjects();
      if (this.currentAccount !== accountAtStart) return;
      this.projects = fresh;
      if (accountAtStart !== null) {
        this.projectsByAccount.set(accountAtStart, fresh);
      }
      // Drop a stale active selection if its project vanished (e.g. its
      // memory dir was deleted out from under us).
      if (
        this.activeProjectId !== null &&
        !fresh.some((p) => p.id === this.activeProjectId)
      ) {
        this.activeProjectId = null;
        this.activeFile = null;
        this.activeFileDirty = false;
      }
    } catch (e) {
      if (this.currentAccount !== accountAtStart) return;
      this.error = e instanceof Error ? e.message : "Failed to load memory projects";
    } finally {
      if (this.currentAccount === accountAtStart) {
        this.loading = false;
      }
    }
  }

  private async loadFilesFor(projectId: string) {
    const accountAtStart = this.currentAccount;
    this.loading = true;
    this.error = "";
    try {
      const files = await ipcClient.listMemoryFiles(projectId);
      // Discard if account flipped — these files belong to a different
      // account's project dir and would corrupt filesByProject under the
      // new account.
      if (this.currentAccount !== accountAtStart) return;
      this.filesByProject = { ...this.filesByProject, [projectId]: files };
    } catch (e) {
      if (this.currentAccount !== accountAtStart) return;
      this.error = e instanceof Error ? e.message : "Failed to load memory files";
    } finally {
      if (this.currentAccount === accountAtStart) {
        this.loading = false;
      }
    }
  }

  /** Toggle a project's expanded state. On first expand, fetches its file
   *  list (cached thereafter). On collapse, the cached list is kept so a
   *  re-expand is instant. */
  async toggleProject(id: string) {
    const next = !this.expanded[id];
    this.expanded = { ...this.expanded, [id]: next };
    if (next && !this.filesByProject[id]) {
      await this.loadFilesFor(id);
    }
  }

  async selectFile(projectId: string, filename: string) {
    const seq = ++this.selectSeq;
    this.loading = true;
    this.error = "";
    try {
      const detail = await ipcClient.getMemoryFile(projectId, filename);
      // A newer selectFile started after us — discard our result so it
      // doesn't clobber theirs (out-of-order IPC responses).
      if (seq !== this.selectSeq) return;
      if (this.currentAccount !== null) {
        this.rememberedProjectByAccount.set(this.currentAccount, projectId);
      }
      this.activeProjectId = projectId;
      this.activeFile = detail;
    } catch (e) {
      if (seq !== this.selectSeq) return;
      this.error = e instanceof Error ? e.message : "Failed to load memory file";
    } finally {
      if (seq === this.selectSeq) {
        this.loading = false;
      }
    }
  }

  async saveFile(projectId: string, filename: string, content: string) {
    const accountAtStart = this.currentAccount;
    this.saving = true;
    this.error = "";
    try {
      await ipcClient.updateMemoryFile(projectId, filename, content);
      if (this.currentAccount !== accountAtStart) return;
      if (this.activeFile && this.activeFile.filename === filename) {
        this.activeFile = { ...this.activeFile, content };
        toastStore.success("File saved");
      }
    } catch (e) {
      if (this.currentAccount !== accountAtStart) return;
      this.error = e instanceof Error ? e.message : "Failed to save memory file";
      toastStore.error(this.error);
    } finally {
      if (this.currentAccount === accountAtStart) {
        this.saving = false;
      }
    }
  }

  async deleteFile(projectId: string, filename: string) {
    const accountAtStart = this.currentAccount;
    this.error = "";
    try {
      await ipcClient.deleteMemoryFile(projectId, filename);
      if (this.currentAccount !== accountAtStart) return;
      const list = this.filesByProject[projectId];
      if (list) {
        this.filesByProject = {
          ...this.filesByProject,
          [projectId]: list.filter((f) => f.filename !== filename),
        };
      }
      // Keep MemoryProject.fileCount roughly in sync so the "hide empty
      // projects" filter reacts when the last file in a project is deleted.
      this.projects = this.projects.map((p) =>
        p.id === projectId ? { ...p, fileCount: Math.max(0, p.fileCount - 1) } : p,
      );
      if (this.activeFile?.filename === filename) {
        this.activeFile = null;
        toastStore.success("File deleted");
      }
    } catch (e) {
      if (this.currentAccount !== accountAtStart) return;
      this.error = e instanceof Error ? e.message : "Failed to delete memory file";
      toastStore.error(this.error);
    }
  }

  reset(): void {
    this.projects = [];
    this.filesByProject = {};
    this.expanded = {};
    this.activeProjectId = null;
    this.activeFile = null;
    this.activeFileDirty = false;
    this.loading = false;
    this.saving = false;
    this.error = "";
    this.selectSeq++;
  }
}

export const memoryStore = new MemoryStore();

import { connectionStore } from "./connection.svelte";
import { toastStore } from "./toast.svelte";
import type { ClaudeMdFile, ClaudeMdFileDetail } from "$lib/api/types";

class ClaudeMdStore {
  files = $state<ClaudeMdFile[]>([]);
  activeFile = $state<ClaudeMdFileDetail | null>(null);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  async loadFiles() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.files = await client.listClaudeMdFiles();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load CLAUDE.md files";
    } finally {
      this.loading = false;
    }
  }

  async loadFile(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.activeFile = await client.getClaudeMdFile(id);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load CLAUDE.md";
      this.activeFile = null;
    } finally {
      this.loading = false;
    }
  }

  async saveFile(id: string, content: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.saving = true;
    this.error = "";
    try {
      await client.updateClaudeMdFile(id, content);
      if (this.activeFile && this.activeFile.id === id) {
        this.activeFile = { ...this.activeFile, content };
      }
      this.files = this.files.map((f) =>
        f.id === id ? { ...f, exists: true } : f
      );
      toastStore.success("CLAUDE.md saved");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save CLAUDE.md";
      toastStore.error(this.error);
    } finally {
      this.saving = false;
    }
  }

  async deleteFile(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.error = "";
    try {
      await client.deleteClaudeMdFile(id);
      this.files = this.files.map((f) =>
        f.id === id ? { ...f, exists: false } : f
      );
      if (this.activeFile?.id === id) {
        this.activeFile = null;
      }
      toastStore.success("CLAUDE.md deleted");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to delete CLAUDE.md";
      toastStore.error(this.error);
    }
  }

  selectFile(id: string) {
    const file = this.files.find((f) => f.id === id);
    if (file && file.exists) {
      void this.loadFile(id);
    } else {
      this.activeFile = {
        id,
        scope: file?.scope ?? "project",
        filename: "CLAUDE.md",
        path: file?.path ?? "",
        content: "",
        projectId: file?.projectId,
      };
    }
  }

  reset(): void {
    this.files = [];
    this.activeFile = null;
    this.loading = false;
    this.saving = false;
    this.error = "";
  }
}

export const claudeMdStore = new ClaudeMdStore();

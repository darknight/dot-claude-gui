import { connectionStore } from "./connection.svelte";
import type { MemoryProject, MemoryFile, MemoryFileDetail } from "$lib/api/types";
import { toastStore } from "./toast.svelte";

class MemoryStore {
  projects = $state<MemoryProject[]>([]);
  activeProjectId = $state<string | null>(null);
  files = $state<MemoryFile[]>([]);
  activeFile = $state<MemoryFileDetail | null>(null);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  async loadProjects() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.projects = await client.listMemoryProjects();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load memory projects";
    } finally {
      this.loading = false;
    }
  }

  async loadFiles(projectId: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.files = await client.listMemoryFiles(projectId);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load memory files";
    } finally {
      this.loading = false;
    }
  }

  async loadFile(projectId: string, filename: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.activeFile = await client.getMemoryFile(projectId, filename);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load memory file";
    } finally {
      this.loading = false;
    }
  }

  async saveFile(projectId: string, filename: string, content: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.saving = true;
    this.error = "";
    try {
      await client.updateMemoryFile(projectId, filename, content);
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
    const client = connectionStore.client;
    if (!client) return;
    this.error = "";
    try {
      await client.deleteMemoryFile(projectId, filename);
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
    this.activeProjectId = id;
    this.files = [];
    this.activeFile = null;
    void this.loadFiles(id);
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
    this.loading = false;
    this.saving = false;
    this.error = "";
  }
}

export const memoryStore = new MemoryStore();

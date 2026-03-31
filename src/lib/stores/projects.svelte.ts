import type { ProjectEntry } from "$lib/api/types";
import { connectionStore } from "./connection.svelte";

class ProjectsStore {
  projects = $state<ProjectEntry[]>([]);
  activeProjectId = $state<string | null>(null);
  loading = $state(false);

  get activeProject(): ProjectEntry | undefined {
    return this.projects.find((p) => p.id === this.activeProjectId);
  }

  async loadProjects(): Promise<void> {
    if (!connectionStore.client) return;
    this.loading = true;
    try {
      this.projects = await connectionStore.client.listProjects();
    } finally {
      this.loading = false;
    }
  }

  async registerProject(path: string): Promise<void> {
    if (!connectionStore.client) return;
    const entry = await connectionStore.client.registerProject(path);
    this.projects = [...this.projects, entry];
  }

  async unregisterProject(id: string): Promise<void> {
    if (!connectionStore.client) return;
    await connectionStore.client.unregisterProject(id);
    this.projects = this.projects.filter((p) => p.id !== id);
    if (this.activeProjectId === id) {
      this.activeProjectId = null;
    }
  }

  selectProject(id: string | null): void {
    this.activeProjectId = id;
  }

  reset(): void {
    this.projects = [];
    this.activeProjectId = null;
    this.loading = false;
  }
}

export const projectsStore = new ProjectsStore();

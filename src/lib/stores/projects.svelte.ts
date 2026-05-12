import { ipcClient } from "$lib/ipc/client";
import type { LaunchConfig, ProjectEntry } from "$lib/api/types";

class ProjectsStore {
  /** Full list from backend; one entry per path in knownProjects. */
  entries = $state<ProjectEntry[]>([]);
  /** Path of the currently focused project (Stage 2 wires UI to this). */
  selectedPath = $state<string | null>(null);

  selected = $derived(
    this.entries.find((e) => e.path === this.selectedPath) ?? null,
  );

  /** Same as `selected` — semantic alias used by project-mode facets. */
  currentBinding = $derived(this.selected);

  /** True only if the project is in `projects` map (has a non-empty account binding). */
  currentBound = $derived(
    this.currentBinding != null && this.currentBinding.account != null && this.currentBinding.account !== "",
  );

  currentStale = $derived(this.currentBinding?.stale === true);

  currentAccount = $derived<string | null>(this.currentBinding?.account ?? null);

  currentLaunch = $derived<LaunchConfig | null>(this.currentBinding?.launch ?? null);

  selectProject(path: string | null): void {
    this.selectedPath = path;
  }

  async loadProjects(): Promise<void> {
    try {
      this.entries = await ipcClient.listProjects();
    } catch {
      this.entries = [];
    }
  }

  async add(path: string): Promise<void> {
    await ipcClient.addProject(path);
    await this.loadProjects();
  }

  async bind(path: string, account: string): Promise<void> {
    await ipcClient.bindProject(path, account);
    await this.loadProjects();
  }

  async unbind(path: string): Promise<void> {
    await ipcClient.unbindProject(path);
    await this.loadProjects();
  }

  async remove(path: string): Promise<void> {
    await ipcClient.removeProject(path);
    await this.loadProjects();
  }

  async updateLaunch(path: string, launch: LaunchConfig): Promise<void> {
    await ipcClient.updateProjectLaunch(path, launch);
    await this.loadProjects();
  }

}

export const projectsStore = new ProjectsStore();

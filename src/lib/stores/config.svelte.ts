import { ipcClient } from "$lib/ipc/client.js";
import { projectsStore } from "./projects.svelte";
import type { Settings } from "$lib/api/types";
import { toastStore } from "./toast.svelte";

class ConfigStore {
  userSettings = $state<Settings>({});
  projectSettings = $state<Settings>({});
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");
  activeScope = $state<"user" | "project">("user");
  isDirty = $state(false);

  /** The settings being edited based on active scope */
  get activeSettings(): Settings {
    return this.activeScope === "project" ? this.projectSettings : this.userSettings;
  }

  async loadUserConfig() {
    this.loading = true;
    this.isDirty = false;
    this.error = "";
    try {
      const res = await ipcClient.getUserConfig();
      this.userSettings = res.settings;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load config";
    } finally {
      this.loading = false;
    }
  }

  async loadProjectConfig(projectId: string) {
    this.loading = true;
    this.isDirty = false;
    this.error = "";
    try {
      const res = await ipcClient.getProjectConfig(projectId);
      this.projectSettings = res.settings;
    } catch {
      // Project may not have a .claude/settings.json yet
      this.projectSettings = {};
    } finally {
      this.loading = false;
    }
  }

  markDirty() {
    this.isDirty = true;
  }

  async save(partialSettings: Partial<Settings>) {
    this.saving = true;
    this.error = "";
    try {
      if (this.activeScope === "project" && projectsStore.activeProjectId) {
        const res = await ipcClient.updateProjectConfig(
          projectsStore.activeProjectId,
          partialSettings,
        );
        this.projectSettings = res.settings;
      } else {
        const res = await ipcClient.updateUserConfig(partialSettings);
        this.userSettings = res.settings;
      }
      this.isDirty = false;
      toastStore.success("Settings saved");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save";
      toastStore.error(this.error);
      throw e;
    } finally {
      this.saving = false;
    }
  }

  setUserConfig(settings: Settings): void {
    this.userSettings = settings;
    this.isDirty = false;
  }

  async revert(): Promise<void> {
    // Re-fetch saved settings from backend so sub-editors' $effect resync
    // their local state. Clearing isDirty alone does not trigger those effects.
    if (this.activeScope === "project" && projectsStore.activeProjectId) {
      await this.loadProjectConfig(projectsStore.activeProjectId);
    } else {
      await this.loadUserConfig();
    }
  }

  reset(): void {
    this.userSettings = {} as Settings;
    this.projectSettings = {} as Settings;
    this.loading = false;
    this.saving = false;
    this.error = "";
    this.activeScope = "user";
    this.isDirty = false;
  }
}

export const configStore = new ConfigStore();

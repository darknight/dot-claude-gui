import { connectionStore } from "./connection.svelte";
import { projectsStore } from "./projects.svelte";
import type { Settings } from "$lib/api/types";

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
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      const res = await client.getUserConfig();
      this.userSettings = res.settings;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load config";
    } finally {
      this.loading = false;
    }
  }

  async loadProjectConfig(projectId: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      const res = await client.getProjectConfig(projectId);
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
    const client = connectionStore.client;
    if (!client) return;
    this.saving = true;
    this.error = "";
    try {
      if (this.activeScope === "project" && projectsStore.activeProjectId) {
        const res = await client.updateProjectConfig(
          projectsStore.activeProjectId,
          partialSettings,
        );
        this.projectSettings = res.settings;
      } else {
        const res = await client.updateUserConfig(partialSettings);
        this.userSettings = res.settings;
      }
      this.isDirty = false;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save";
      throw e;
    } finally {
      this.saving = false;
    }
  }

  revert() {
    this.isDirty = false;
    this.error = "";
    // Trigger re-render by re-loading from cache
    // The sub-editors will re-sync from activeSettings via $effect
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

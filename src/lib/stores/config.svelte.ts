import { ipcClient } from "$lib/ipc/client.js";
import type { Settings } from "$lib/api/types";
import { toastStore } from "./toast.svelte";

class ConfigStore {
  userSettings = $state<Settings>({});
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");
  isDirty = $state(false);

  /** The settings being edited. */
  get activeSettings(): Settings {
    return this.userSettings;
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

  markDirty() {
    this.isDirty = true;
  }

  async save(partialSettings: Partial<Settings>) {
    this.saving = true;
    this.error = "";
    try {
      const res = await ipcClient.updateUserConfig(partialSettings);
      this.userSettings = res.settings;
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
    await this.loadUserConfig();
  }

  reset(): void {
    this.userSettings = {} as Settings;
    this.loading = false;
    this.saving = false;
    this.error = "";
    this.isDirty = false;
  }
}

export const configStore = new ConfigStore();

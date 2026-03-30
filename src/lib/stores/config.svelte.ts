import type { Settings } from "$lib/api/types";
import { connectionStore } from "./connection.svelte";

class ConfigStore {
  userSettings = $state<Settings>({});
  loading = $state(false);
  error = $state<string>("");

  async loadUserConfig(): Promise<void> {
    if (!connectionStore.client) return;
    this.loading = true;
    this.error = "";
    try {
      const response = await connectionStore.client.getUserConfig();
      this.userSettings = response.settings;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.loading = false;
    }
  }

  async updateUserConfig(settings: Partial<Settings>): Promise<void> {
    if (!connectionStore.client) return;
    this.loading = true;
    this.error = "";
    try {
      const merged: Settings = { ...this.userSettings, ...settings };
      const response = await connectionStore.client.updateUserConfig(merged);
      this.userSettings = response.settings;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.loading = false;
    }
  }
}

export const configStore = new ConfigStore();

import type { AppConfig } from "$lib/api/types.js";
import { invoke } from "@tauri-apps/api/core";
import { detectInitialLocale, isSupportedLocale } from "$lib/i18n";

class AppSettingsStore {
  preferences = $state<AppConfig>({
    schemaVersion: 2,
    theme: "system",
    language: "zh-CN",
    fontSize: 14,
    sidebarWidth: 140,
    preferredTerminal: "terminal",
    accounts: [],
    projects: {},
    knownProjects: [],
  });

  loaded = $state(false);

  async load(): Promise<void> {
    try {
      const json = await invoke<string>("read_app_config");
      const saved: Partial<AppConfig> = JSON.parse(json);
      this.preferences = { ...this.preferences, ...saved };
    } catch {
      // defaults
    }

    if (!isSupportedLocale(this.preferences.language)) {
      this.preferences.language = detectInitialLocale();
      await this.save();
    }
    this.loaded = true;
  }

  async save(): Promise<void> {
    try {
      await invoke("write_app_config", {
        json: JSON.stringify(this.preferences, null, 2),
      });
    } catch {}
  }

  async update(partial: Partial<AppConfig>): Promise<void> {
    this.preferences = { ...this.preferences, ...partial };
    await this.save();
  }
}

export const appSettingsStore = new AppSettingsStore();

import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "$lib/api/types.js";
import { detectInitialLocale, isSupportedLocale } from "$lib/i18n";

class AppSettingsStore {
  preferences = $state<AppConfig>({
    theme: "system",
    language: undefined,
    fontSize: 14,
    sidebarWidth: 140,
    subpanelWidth: 240,
  });

  loaded = $state(false);

  async load(): Promise<void> {
    try {
      const json = await invoke<string>("read_app_config");
      const saved: Partial<AppConfig> = JSON.parse(json);
      this.preferences = { ...this.preferences, ...saved };
    } catch {
      // Use defaults on error
    }

    // One-time migration from localStorage
    try {
      const legacy = localStorage.getItem("dot-claude-gui-preferences");
      if (legacy) {
        const parsed = JSON.parse(legacy);
        if (parsed.theme) this.preferences.theme = parsed.theme;
        if (parsed.language) this.preferences.language = parsed.language;
        if (parsed.fontSize) this.preferences.fontSize = parsed.fontSize;
        await this.save();
        localStorage.removeItem("dot-claude-gui-preferences");
      }
    } catch {
      // Ignore migration errors
    }

    // Dirty-data self-heal + cold-start detection
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
    } catch {
      // Silently fail — preferences are not critical
    }
  }

  async update(partial: Partial<AppConfig>): Promise<void> {
    this.preferences = { ...this.preferences, ...partial };
    await this.save();
  }
}

export const appSettingsStore = new AppSettingsStore();

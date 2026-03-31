interface AppPreferences {
  theme: "light" | "dark" | "system";
  language: string;
  fontSize: number;
  daemonUrl: string;
  daemonToken: string;
}

class AppSettingsStore {
  preferences = $state<AppPreferences>({
    theme: "system",
    language: "zh-CN",
    fontSize: 14,
    daemonUrl: "http://127.0.0.1:7890",
    daemonToken: "dev-token",
  });

  load() {
    try {
      const saved = localStorage.getItem("dot-claude-gui-preferences");
      if (saved) {
        this.preferences = { ...this.preferences, ...JSON.parse(saved) };
      }
    } catch { /* ignore parse errors */ }
  }

  save() {
    localStorage.setItem("dot-claude-gui-preferences", JSON.stringify(this.preferences));
  }

  update(partial: Partial<AppPreferences>) {
    this.preferences = { ...this.preferences, ...partial };
    this.save();
  }
}

export const appSettingsStore = new AppSettingsStore();

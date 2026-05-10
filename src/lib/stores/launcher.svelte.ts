import { ipcClient } from "$lib/ipc/client";
import { appSettingsStore } from "./appsettings.svelte";
import { parseClaudeHelp, type ClaudeArg } from "$lib/data/parseClaudeHelp";
import type { LauncherEnvEntry, LauncherArgEntry } from "$lib/api/types";

class LauncherStore {
  selectedProjectId = $state<string>("");
  customEnv = $state<LauncherEnvEntry[]>([]);
  customArgs = $state<LauncherArgEntry[]>([]);
  accountName = $state<string | undefined>(undefined);
  claudeArgs = $state<ClaudeArg[]>([]);
  claudeArgsLoaded = $state<boolean>(false);

  selectProject(id: string): void {
    this.selectedProjectId = id;
  }

  /** Load persisted env/args for a project path. Call when selection changes. */
  loadForProject(projectPath: string): void {
    const map = appSettingsStore.preferences.launcherProjectEnv ?? {};
    const saved = map[projectPath];
    this.customEnv = saved?.customEnv ? [...saved.customEnv] : [];
    this.customArgs = saved?.customArgs ? [...saved.customArgs] : [];
    this.accountName = saved?.accountName;
  }

  /** Persist current state for the given project path. */
  async persistFor(projectPath: string): Promise<void> {
    const map = { ...(appSettingsStore.preferences.launcherProjectEnv ?? {}) };
    map[projectPath] = {
      customEnv: $state.snapshot(this.customEnv),
      customArgs: $state.snapshot(this.customArgs),
      accountName: this.accountName,
    };
    await appSettingsStore.update({ launcherProjectEnv: map });
  }

  async setAccount(projectPath: string, name: string | undefined): Promise<void> {
    this.accountName = name;
    await this.persistFor(projectPath);
  }

  /** Pull `claude --help` and parse it. Failure leaves the list empty. */
  async loadClaudeArgs(): Promise<void> {
    try {
      const stdout = await ipcClient.getClaudeArgs();
      this.claudeArgs = parseClaudeHelp(stdout);
    } catch {
      this.claudeArgs = [];
    } finally {
      this.claudeArgsLoaded = true;
    }
  }

  // ---- env mutations ----

  async addCustomVar(projectPath: string, key: string, value: string): Promise<void> {
    this.customEnv = [...this.customEnv, { key, value, enabled: true }];
    await this.persistFor(projectPath);
  }

  async removeCustomVar(projectPath: string, index: number): Promise<void> {
    this.customEnv = this.customEnv.filter((_, i) => i !== index);
    await this.persistFor(projectPath);
  }

  async setCustomVarEnabled(projectPath: string, index: number, enabled: boolean): Promise<void> {
    this.customEnv = this.customEnv.map((e, i) => (i === index ? { ...e, enabled } : e));
    await this.persistFor(projectPath);
  }

  // ---- args mutations ----

  async addCustomArg(projectPath: string, flag: string, value: string | undefined): Promise<void> {
    this.customArgs = [...this.customArgs, { flag, value, enabled: true }];
    await this.persistFor(projectPath);
  }

  async removeCustomArg(projectPath: string, index: number): Promise<void> {
    this.customArgs = this.customArgs.filter((_, i) => i !== index);
    await this.persistFor(projectPath);
  }

  async setArgEnabled(projectPath: string, index: number, enabled: boolean): Promise<void> {
    this.customArgs = this.customArgs.map((a, i) => (i === index ? { ...a, enabled } : a));
    await this.persistFor(projectPath);
  }

  async setArgValue(projectPath: string, index: number, value: string): Promise<void> {
    this.customArgs = this.customArgs.map((a, i) => (i === index ? { ...a, value } : a));
    await this.persistFor(projectPath);
  }
}

export const launcherStore = new LauncherStore();

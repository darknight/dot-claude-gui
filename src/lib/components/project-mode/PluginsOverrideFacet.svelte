<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { runStreamingCommand } from "$lib/ipc/events";
  import type { PluginInfo, Settings } from "$lib/api/types";

  let { path }: { path: string } = $props();

  let uninstallingId = $state<string | null>(null);
  let outputLines = $state<string[]>([]);

  function accountLabel(name: string): string {
    const acc = accountsStore.accounts.find((a) => a.name === name);
    return acc?.displayName || name;
  }

  type Tri = "inherit" | "enable" | "disable";

  let plugins = $state<PluginInfo[]>([]);
  let projectSettings = $state<Settings>({});
  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);

  async function load() {
    loading = true;
    error = null;
    try {
      const [list, settingsResp] = await Promise.all([
        ipcClient.projectListPlugins(path),
        ipcClient.projectReadSettings(path),
      ]);
      plugins = list;
      projectSettings = settingsResp.settings ?? {};
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function stateOf(id: string): Tri {
    const map = projectSettings.enabledPlugins;
    if (!map || !(id in map)) return "inherit";
    return map[id] ? "enable" : "disable";
  }

  /// Bound account name for this project (from app config). Foreign rows
  /// override this with their own foreignAccount; otherwise we route the
  /// uninstall through the bound account so the CLI sees the same
  /// installed_plugins.json the listing came from.
  function boundAccount(): string | undefined {
    return appSettingsStore.preferences.projects?.[path]?.account;
  }

  async function handleUninstall(p: PluginInfo) {
    outputLines = [];
    uninstallingId = p.id;

    const accountName = p.foreignAccount ?? boundAccount();
    // Project-scope needs cwd (so the CLI is inside the owning project)
    // and `--scope project` (so it doesn't default to user scope and
    // reject with the misleading "enabled at project scope" error).
    const cwd = p.scope === "project" ? p.projectPath ?? path : undefined;

    const ok = await runStreamingCommand(
      () => pluginsStore.uninstallPlugin(p.id, { accountName, cwd, scope: p.scope }),
      (line) => { outputLines = [...outputLines, line]; },
      async (exitCode) => {
        uninstallingId = null;
        if (exitCode === 0) {
          toastStore.success(t("plugins.uninstallSuccess"));
          outputLines = [];
          await load();
        } else {
          toastStore.error(t("plugins.uninstallFailed", { exitCode }));
        }
      },
    );
    if (!ok) {
      uninstallingId = null;
      toastStore.error(
        t("plugins.uninstallError", { msg: pluginsStore.error || "unknown" }),
      );
    }
  }

  async function setState(id: string, next: Tri) {
    saving = true;
    try {
      const cur: Record<string, boolean> = { ...(projectSettings.enabledPlugins ?? {}) };
      if (next === "inherit") {
        delete cur[id];
      } else {
        cur[id] = next === "enable";
      }
      const updated: Settings = {
        ...projectSettings,
        enabledPlugins: Object.keys(cur).length > 0 ? cur : undefined,
      };
      await ipcClient.projectWriteSettings(path, updated);
      projectSettings = updated;
      toastStore.success(t("projectMode.plugins.saved"));
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }
</script>

<section class="plugins-facet">
  <h2>{t("projectMode.plugins.title")}</h2>
  <p class="hint">{t("projectMode.plugins.hint")}</p>

  {#if loading}
    <div class="empty">{t("projectMode.plugins.loading")}</div>
  {:else if error}
    <div class="err">{error}</div>
  {:else if plugins.length === 0}
    <div class="empty">{t("projectMode.plugins.noPluginsAccount")}</div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("projectMode.plugins.name")}</th>
          <th>{t("projectMode.plugins.version")}</th>
          <th>{t("projectMode.plugins.override")}</th>
          <th class="actions-th">{t("projectMode.plugins.actions")}</th>
        </tr>
      </thead>
      <tbody>
        {#each plugins as p (p.id + ":" + (p.foreignAccount ?? "") + ":" + (p.projectPath ?? ""))}
          {@const st = stateOf(p.id)}
          <tr>
            <td>
              <div class="plugin-name">
                {p.name}
                {#if p.foreignAccount}
                  <span class="badge-foreign" title={t("projectMode.plugins.foreignHint", { account: accountLabel(p.foreignAccount) })}>
                    {t("projectMode.plugins.foreignBadge", { account: accountLabel(p.foreignAccount) })}
                  </span>
                {/if}
              </div>
              <div class="plugin-meta">{p.marketplace}</div>
            </td>
            <td>{p.version}</td>
            <td>
              <div class="tri" role="radiogroup" aria-label={t("projectMode.plugins.override")}>
                <button
                  type="button"
                  class:active={st === "disable"}
                  disabled={saving}
                  onclick={() => setState(p.id, "disable")}
                >{t("projectMode.plugins.disable")}</button>
                <button
                  type="button"
                  class:active={st === "inherit"}
                  disabled={saving}
                  onclick={() => setState(p.id, "inherit")}
                >{t("projectMode.plugins.inherit")}</button>
                <button
                  type="button"
                  class:active={st === "enable"}
                  disabled={saving}
                  onclick={() => setState(p.id, "enable")}
                >{t("projectMode.plugins.enable")}</button>
              </div>
            </td>
            <td>
              <button
                type="button"
                class="uninstall-btn"
                disabled={uninstallingId !== null}
                title={p.scope === "project"
                  ? t("plugins.uninstallProjectTitle", { path: p.projectPath ?? path })
                  : t("plugins.uninstall")}
                onclick={() => handleUninstall(p)}
              >
                {uninstallingId === p.id
                  ? t("plugins.uninstalling")
                  : t("plugins.uninstall")}
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if outputLines.length > 0}
      <pre class="cli-output">{outputLines.join("\n")}</pre>
    {/if}
  {/if}
</section>

<style>
  .plugins-facet { padding: 16px; color: var(--text-primary); }
  h2 { margin: 0 0 8px; font-size: 16px; font-weight: 600; }
  .hint { color: var(--text-muted); font-size: 12px; margin: 0 0 16px; }
  table { width: 100%; border-collapse: collapse; }
  th, td {
    text-align: left;
    padding: 8px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }
  th { color: var(--text-muted); font-weight: 600; font-size: 12px; }
  .plugin-name { font-weight: 500; }
  .plugin-meta { font-size: 11px; color: var(--text-muted); margin-top: 2px; }
  .badge-foreign {
    display: inline-block;
    margin-left: 6px;
    padding: 1px 6px;
    font-size: 10px;
    font-weight: 500;
    border-radius: 3px;
    background-color: var(--status-warning-bg);
    color: var(--status-warning-text);
    vertical-align: middle;
  }
  .tri {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
  }
  .tri button {
    background: transparent;
    border: none;
    border-right: 1px solid var(--border);
    padding: 4px 10px;
    color: inherit;
    cursor: pointer;
    font-size: 12px;
  }
  .tri button:last-child { border-right: none; }
  .tri button.active {
    background: var(--accent, #2c6cff);
    color: white;
  }
  .tri button[disabled] { opacity: 0.6; cursor: not-allowed; }
  .actions-th { width: 1%; white-space: nowrap; text-align: right; }
  td:last-child { width: 1%; white-space: nowrap; text-align: right; }
  .uninstall-btn {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 10px;
    font-size: 12px;
    color: var(--danger, #c44);
    cursor: pointer;
    white-space: nowrap;
  }
  .uninstall-btn:hover:not([disabled]) { background-color: var(--status-error-bg); }
  .uninstall-btn[disabled] { opacity: 0.5; cursor: not-allowed; }
  .cli-output {
    margin-top: 12px;
    max-height: 200px;
    overflow: auto;
    padding: 8px 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, monospace;
    font-size: 11px;
    color: var(--text-secondary);
    background-color: var(--bg-secondary, #f5f5f5);
    border: 1px solid var(--border);
    border-radius: 4px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .empty, .err {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
  .err { color: var(--danger, #c44); }
</style>

<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { PluginInfo, Settings } from "$lib/api/types";

  let { path }: { path: string } = $props();

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
        </tr>
      </thead>
      <tbody>
        {#each plugins as p (p.id)}
          {@const st = stateOf(p.id)}
          <tr>
            <td>
              <div class="plugin-name">{p.name}</div>
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
          </tr>
        {/each}
      </tbody>
    </table>
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
  .empty, .err {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
  .err { color: var(--danger, #c44); }
</style>

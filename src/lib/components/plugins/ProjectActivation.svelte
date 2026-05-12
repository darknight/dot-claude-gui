<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { ipcClient } from "$lib/ipc/client.js";
  import type { Settings } from "$lib/api/types";
  import { t } from "$lib/i18n";

  let projectSettings = $state<Settings>({});
  let loading = $state(false);
  let saving = $state<string | null>(null); // pluginId being saved

  const activeProject = $derived(projectsStore.activeProject);
  const activeProjectName = $derived(
    activeProject?.path.replace(/^.*\//, "") ?? "",
  );

  $effect(() => {
    if (activeProject) {
      loadProjectSettings(activeProject.path);
    } else {
      projectSettings = {};
    }
  });

  async function loadProjectSettings(projectId: string) {
    loading = true;
    try {
      const res = await ipcClient.getProjectConfig(projectId);
      projectSettings = res.settings ?? {};
    } catch {
      projectSettings = {};
    } finally {
      loading = false;
    }
  }

  /**
   * Returns true/false if there's an explicit project override,
   * or null if the project doesn't override this plugin.
   *
   * `enabledPlugins` is a `Record<string, boolean>` map.
   * Missing key → null (inherit); value `true` → enable; `false` → disable.
   */
  function getProjectOverride(pluginId: string): boolean | null {
    const map = projectSettings.enabledPlugins;
    if (!map || !(pluginId in map)) return null;
    return map[pluginId];
  }

  async function toggleGlobal(pluginId: string, enabled: boolean) {
    await pluginsStore.togglePlugin(pluginId, enabled);
  }

  async function toggleProjectPlugin(pluginId: string, nextState: boolean | null) {
    if (!activeProject) return;
    saving = pluginId;
    try {
      const cur: Record<string, boolean> = { ...(projectSettings.enabledPlugins ?? {}) };
      if (nextState === null) {
        delete cur[pluginId];
      } else {
        cur[pluginId] = nextState;
      }
      const next = Object.keys(cur).length === 0 ? undefined : cur;
      const res = await ipcClient.updateProjectConfig(activeProject.path, {
        enabledPlugins: next,
      });
      projectSettings = res.settings ?? {};
    } catch {
      /* ignore errors */
    } finally {
      saving = null;
    }
  }

  /**
   * Cycle through states for project checkbox:
   * inherit (null) → enabled (true) → disabled (false) → inherit (null)
   */
  function cycleProjectState(pluginId: string) {
    const current = getProjectOverride(pluginId);
    if (current === null) {
      toggleProjectPlugin(pluginId, true);
    } else if (current === true) {
      toggleProjectPlugin(pluginId, false);
    } else {
      // false → back to inherit (remove override)
      toggleProjectPlugin(pluginId, null);
    }
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !activeProject}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">{t("plugins.selectProjectHint")}</p>
    </div>
  {:else}
    <div class="flex-1 overflow-auto p-6">
      <div class="mb-4">
        <h2 class="text-sm font-semibold" style="color: var(--text-primary)">{t("plugins.activationTitle")}</h2>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">
          {t("plugins.activationDesc")}
          {t("plugins.activationProjectCol")}
          <span style="color: var(--status-success-text)">{t("plugins.stateOn")}</span> /
          <span style="color: var(--status-error-text)">{t("plugins.stateOff")}</span> /
          <span style="color: var(--text-muted)">{t("plugins.stateInherit")}</span>
          {t("plugins.activationCycleHint")}
        </p>
      </div>

      {#if loading}
        <p class="text-sm" style="color: var(--text-muted)">{t("plugins.loadingProjectSettings")}</p>
      {:else if pluginsStore.plugins.length === 0}
        <div class="flex h-40 items-center justify-center">
          <p class="text-sm" style="color: var(--text-muted)">{t("plugins.noPlugins")}</p>
        </div>
      {:else}
        <div class="overflow-hidden rounded-lg" style="border: 1px solid var(--border-color)">
          <table class="w-full text-sm">
            <thead>
              <tr style="border-bottom: 1px solid var(--border-color); background-color: var(--bg-card)">
                <th class="px-4 py-2.5 text-left text-xs font-medium uppercase tracking-wider" style="color: var(--text-muted)">
                  {t("plugins.colPlugin")}
                </th>
                <th class="w-24 px-4 py-2.5 text-center text-xs font-medium uppercase tracking-wider" style="color: var(--text-muted)">
                  {t("plugins.colGlobal")}
                </th>
                <th class="w-32 px-4 py-2.5 text-center text-xs font-medium uppercase tracking-wider" style="color: var(--text-muted)">
                  {activeProjectName}
                </th>
              </tr>
            </thead>
            <tbody style="background-color: var(--bg-card)">
              {#each pluginsStore.plugins as plugin (plugin.id)}
                {@const projectOverride = getProjectOverride(plugin.id)}
                {@const isSaving = saving === plugin.id}
                <tr class="transition-colors hover:bg-[var(--bg-card-hover)]" style="border-top: 1px solid var(--border-subtle)">
                  <!-- Plugin name + info -->
                  <td class="px-4 py-3">
                    <div class="flex items-center gap-2">
                      <span class="font-medium" style="color: var(--text-primary)">{plugin.name}</span>
                      {#if plugin.blocked}
                        <span class="badge badge-error">
                          {t("plugins.blocked")}
                        </span>
                      {/if}
                    </div>
                    <div class="mt-0.5 text-xs" style="color: var(--text-muted)">
                      {plugin.marketplace} · v{plugin.version}
                    </div>
                  </td>

                  <!-- Global toggle -->
                  <td class="px-4 py-3 text-center">
                    <button
                      class="toggle-track"
                      role="switch"
                      aria-checked={plugin.enabled}
                      aria-label={t("plugins.toggleGloballyAriaLabel", { name: plugin.name })}
                      onclick={() => toggleGlobal(plugin.id, !plugin.enabled)}
                    >
                      <span class="toggle-knob"></span>
                    </button>
                  </td>

                  <!-- Project override -->
                  <td class="px-4 py-3 text-center">
                    <button
                      class="inline-flex items-center justify-center gap-1.5 rounded px-2.5 py-1 text-xs font-medium transition-colors focus:outline-none disabled:opacity-50"
                      style={projectOverride === true
                        ? 'background-color: var(--status-success-bg); color: var(--status-success-text)'
                        : projectOverride === false
                          ? 'background-color: var(--status-error-bg); color: var(--status-error-text)'
                          : 'background-color: var(--badge-bg); color: var(--badge-text)'}
                      onclick={() => cycleProjectState(plugin.id)}
                      disabled={isSaving}
                      title={projectOverride === true
                        ? t("plugins.titleEnabled")
                        : projectOverride === false
                          ? t("plugins.titleDisabled")
                          : t("plugins.titleInherit")}
                    >
                      {#if isSaving}
                        <span class="h-3 w-3 animate-spin rounded-full border border-current border-t-transparent"></span>
                      {:else if projectOverride === true}
                        <svg class="h-3 w-3" viewBox="0 0 12 12" fill="currentColor">
                          <path d="M10 3L5 8.5 2 5.5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                        {t("plugins.stateOn")}
                      {:else if projectOverride === false}
                        <svg class="h-3 w-3" viewBox="0 0 12 12" fill="currentColor">
                          <path d="M9 3L3 9M3 3l6 6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/>
                        </svg>
                        {t("plugins.stateOff")}
                      {:else}
                        <span class="text-[10px]">—</span>
                        {t("plugins.stateInherit")}
                      {/if}
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <p class="mt-3 text-xs" style="color: var(--text-muted)">
          Project: <strong style="color: var(--text-secondary)">{activeProject.path}</strong>
        </p>
      {/if}
    </div>
  {/if}
</div>

<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { ipcClient } from "$lib/ipc/client.js";
  import type { Settings } from "$lib/api/types";

  let projectSettings = $state<Settings>({});
  let loading = $state(false);
  let saving = $state<string | null>(null); // pluginId being saved

  const activeProject = $derived(projectsStore.activeProject);

  $effect(() => {
    if (activeProject) {
      loadProjectSettings(activeProject.id);
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
   * Since enabledPlugins is a string[] (allowlist), a project override
   * is represented by the presence of the array itself.
   * We treat: array exists + id in array → true, array exists + id absent → false,
   * array absent → null (inherit).
   */
  function getProjectOverride(pluginId: string): boolean | null {
    const arr = projectSettings.enabledPlugins;
    if (!arr) return null; // no project-level setting at all
    return arr.includes(pluginId);
  }

  async function toggleGlobal(pluginId: string, enabled: boolean) {
    await pluginsStore.togglePlugin(pluginId, enabled);
  }

  async function toggleProjectPlugin(pluginId: string, nextState: boolean | null) {
    if (!activeProject) return;
    saving = pluginId;
    try {
      let newEnabledPlugins: string[] | undefined;
      if (nextState === null) {
        // Remove override: clear this plugin from the list.
        // If the list would become empty and represents "no override", set to undefined.
        const current = projectSettings.enabledPlugins ?? [];
        const updated = current.filter((id) => id !== pluginId);
        // Keep the array (even empty) to preserve other overrides; set to undefined only if never set
        newEnabledPlugins = updated.length === 0 && !projectSettings.enabledPlugins ? undefined : updated;
      } else if (nextState === true) {
        // Add to allowlist
        const current = projectSettings.enabledPlugins ?? [];
        if (!current.includes(pluginId)) {
          newEnabledPlugins = [...current, pluginId];
        } else {
          newEnabledPlugins = current;
        }
      } else {
        // nextState === false: explicitly disable — remove from allowlist (keep array to signal override)
        const current = projectSettings.enabledPlugins ?? [];
        newEnabledPlugins = current.filter((id) => id !== pluginId);
      }
      const res = await ipcClient.updateProjectConfig(activeProject.id, {
        enabledPlugins: newEnabledPlugins,
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
      <p class="text-sm text-gray-500">Select a project to configure per-project plugin activation.</p>
    </div>
  {:else}
    <div class="flex-1 overflow-auto p-6">
      <div class="mb-4">
        <h2 class="text-sm font-semibold text-gray-200">Plugin Activation</h2>
        <p class="mt-1 text-xs text-gray-500">
          Configure plugins globally or override per project.
          Project column: <span class="text-green-400">On</span> /
          <span class="text-red-400">Off</span> /
          <span class="text-gray-500">Inherit</span> (click to cycle).
        </p>
      </div>

      {#if loading}
        <p class="text-sm text-gray-500">Loading project settings...</p>
      {:else if pluginsStore.plugins.length === 0}
        <div class="flex h-40 items-center justify-center">
          <p class="text-sm text-gray-600">No plugins installed</p>
        </div>
      {:else}
        <div class="overflow-hidden rounded-lg border border-gray-800">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-gray-800 bg-gray-900">
                <th class="px-4 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                  Plugin
                </th>
                <th class="w-24 px-4 py-2.5 text-center text-xs font-medium uppercase tracking-wider text-gray-500">
                  Global
                </th>
                <th class="w-32 px-4 py-2.5 text-center text-xs font-medium uppercase tracking-wider text-gray-500">
                  {activeProject.name}
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-800/50 bg-gray-900/50">
              {#each pluginsStore.plugins as plugin (plugin.id)}
                {@const projectOverride = getProjectOverride(plugin.id)}
                {@const isSaving = saving === plugin.id}
                <tr class="transition-colors hover:bg-gray-800/30">
                  <!-- Plugin name + info -->
                  <td class="px-4 py-3">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-gray-200">{plugin.name}</span>
                      {#if plugin.blocked}
                        <span class="rounded bg-red-900/60 px-1.5 py-0.5 text-xs font-medium text-red-400">
                          Blocked
                        </span>
                      {/if}
                    </div>
                    <div class="mt-0.5 text-xs text-gray-600">
                      {plugin.marketplace} · v{plugin.version}
                    </div>
                  </td>

                  <!-- Global toggle -->
                  <td class="px-4 py-3 text-center">
                    <button
                      class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none
                        {plugin.enabled ? 'bg-green-600' : 'bg-gray-700'}"
                      role="switch"
                      aria-checked={plugin.enabled}
                      aria-label="Toggle {plugin.name} globally"
                      onclick={() => toggleGlobal(plugin.id, !plugin.enabled)}
                    >
                      <span
                        class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out
                          {plugin.enabled ? 'translate-x-4' : 'translate-x-0'}"
                      ></span>
                    </button>
                  </td>

                  <!-- Project override -->
                  <td class="px-4 py-3 text-center">
                    <button
                      class="inline-flex items-center justify-center gap-1.5 rounded px-2.5 py-1 text-xs font-medium transition-colors focus:outline-none disabled:opacity-50
                        {projectOverride === true
                          ? 'bg-green-900/50 text-green-400 hover:bg-green-900/70'
                          : projectOverride === false
                            ? 'bg-red-900/50 text-red-400 hover:bg-red-900/70'
                            : 'bg-gray-800 text-gray-500 hover:bg-gray-700 hover:text-gray-400'}"
                      onclick={() => cycleProjectState(plugin.id)}
                      disabled={isSaving}
                      title={projectOverride === true
                        ? 'Project: Enabled — click to disable'
                        : projectOverride === false
                          ? 'Project: Disabled — click to inherit'
                          : 'Inheriting global — click to enable'}
                    >
                      {#if isSaving}
                        <span class="h-3 w-3 animate-spin rounded-full border border-current border-t-transparent"></span>
                      {:else if projectOverride === true}
                        <svg class="h-3 w-3" viewBox="0 0 12 12" fill="currentColor">
                          <path d="M10 3L5 8.5 2 5.5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                        On
                      {:else if projectOverride === false}
                        <svg class="h-3 w-3" viewBox="0 0 12 12" fill="currentColor">
                          <path d="M9 3L3 9M3 3l6 6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/>
                        </svg>
                        Off
                      {:else}
                        <span class="text-[10px]">—</span>
                        Inherit
                      {/if}
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <p class="mt-3 text-xs text-gray-600">
          Project: <strong class="text-gray-400">{activeProject.path}</strong>
        </p>
      {/if}
    </div>
  {/if}
</div>

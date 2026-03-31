<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";

  async function handleUninstall(id: string) {
    await pluginsStore.uninstallPlugin(id);
    await pluginsStore.loadPlugins();
  }
</script>

<div class="flex-1 overflow-auto p-6">
  {#if pluginsStore.loading}
    <p class="text-sm text-gray-500">Loading plugins...</p>
  {:else if pluginsStore.error}
    <div class="mb-4 rounded border border-red-800 bg-red-950 px-4 py-2">
      <p class="text-xs text-red-400">{pluginsStore.error}</p>
    </div>
  {/if}

  {#if pluginsStore.plugins.length === 0 && !pluginsStore.loading}
    <div class="flex h-full items-center justify-center">
      <p class="text-sm text-gray-600">No plugins installed</p>
    </div>
  {:else}
    <div class="space-y-3">
      {#each pluginsStore.plugins as plugin (plugin.id)}
        <div class="group relative rounded-lg border border-gray-800 bg-gray-900 px-4 py-3 transition-colors hover:border-gray-700">
          <div class="flex items-start justify-between gap-4">
            <!-- Plugin info -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="font-semibold text-gray-100">{plugin.name}</span>
                {#if plugin.blocked}
                  <span class="rounded bg-red-900 px-1.5 py-0.5 text-xs font-medium text-red-300">
                    Blocked
                  </span>
                {/if}
              </div>
              <div class="mt-0.5 flex items-center gap-2 text-xs text-gray-500">
                <span>{plugin.marketplace}</span>
                <span>·</span>
                <span>v{plugin.version}</span>
              </div>
              {#if plugin.description}
                <p class="mt-1 text-xs text-gray-400">{plugin.description}</p>
              {/if}
            </div>

            <!-- Controls -->
            <div class="flex items-center gap-3">
              <!-- Uninstall button (hover only) -->
              <button
                class="hidden rounded px-2 py-1 text-xs text-gray-500 transition-colors hover:bg-red-900/50 hover:text-red-400 group-hover:block"
                onclick={() => handleUninstall(plugin.id)}
                title="Uninstall plugin"
              >
                Uninstall
              </button>

              <!-- Toggle switch -->
              <button
                class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none
                  {plugin.enabled ? 'bg-green-600' : 'bg-gray-700'}"
                role="switch"
                aria-checked={plugin.enabled}
                aria-label="Toggle {plugin.name}"
                onclick={() => pluginsStore.togglePlugin(plugin.id, !plugin.enabled)}
              >
                <span
                  class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out
                    {plugin.enabled ? 'translate-x-4' : 'translate-x-0'}"
                ></span>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events.js";

  let selectedMarketplace = $state("");
  let installing = $state<string | null>(null); // plugin name being installed
  let installOutput = $state<string[]>([]);

  $effect(() => {
    pluginsStore.loadMarketplaces();
  });

  $effect(() => {
    if (selectedMarketplace) {
      pluginsStore.loadMarketplacePlugins(selectedMarketplace);
    }
  });

  async function handleInstall(name: string, marketplace: string) {
    installing = name;
    installOutput = [];
    const result = await pluginsStore.installPlugin(name, marketplace);
    if (!result) return;

    // Listen for output via Tauri IPC events
    const unsubOutput = await onCommandOutput((p) => {
      if (p.commandId === result.requestId) {
        installOutput = [...installOutput, p.line];
      }
    });
    const unsubCompleted = await onCommandCompleted((p) => {
      if (p.commandId === result.requestId) {
        installing = null;
        pluginsStore.loadPlugins();
        pluginsStore.loadMarketplacePlugins(selectedMarketplace);
        unsubOutput();
        unsubCompleted();
      }
    });
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden p-6">
  <!-- Marketplace selector -->
  <div class="mb-4">
    <label for="marketplace-select" class="mb-1 block text-xs font-medium text-gray-400">
      Marketplace
    </label>
    {#if pluginsStore.marketplaces.length === 0}
      <p class="text-sm text-gray-600">No marketplaces registered. Add one in the Manage Marketplaces tab.</p>
    {:else}
      <select
        id="marketplace-select"
        bind:value={selectedMarketplace}
        class="w-full rounded border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-100 focus:border-blue-500 focus:outline-none"
      >
        <option value="">— select a marketplace —</option>
        {#each pluginsStore.marketplaces as mp (mp.id)}
          <option value={mp.id}>{mp.id} ({mp.repo})</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- Plugin list -->
  {#if selectedMarketplace}
    {#if pluginsStore.availablePlugins.length === 0}
      <div class="flex flex-1 items-center justify-center">
        <p class="text-sm text-gray-600">No plugins found in this marketplace.</p>
      </div>
    {:else}
      <div class="flex-1 overflow-auto">
        <div class="space-y-3">
          {#each pluginsStore.availablePlugins as plugin (plugin.name)}
            <div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3 transition-colors hover:border-gray-700">
              <div class="flex items-start justify-between gap-4">
                <!-- Plugin info -->
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="font-semibold text-gray-100">{plugin.name}</span>
                    {#if plugin.version}
                      <span class="text-xs text-gray-500">v{plugin.version}</span>
                    {/if}
                    {#if plugin.category}
                      <span class="rounded bg-blue-900 px-1.5 py-0.5 text-xs font-medium text-blue-300 dark:bg-blue-900 dark:text-blue-300">
                        {plugin.category}
                      </span>
                    {/if}
                    {#if plugin.installed && plugin.installedVersion}
                      <span class="rounded bg-green-100 px-1.5 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900 dark:text-green-300">
                        ✓ v{plugin.installedVersion}
                      </span>
                    {/if}
                  </div>
                  {#if plugin.description}
                    <p class="mt-1 text-xs text-gray-400">{plugin.description}</p>
                  {/if}
                </div>

                <!-- Action -->
                <div class="flex flex-shrink-0 items-center gap-2">
                  {#if installing === plugin.name}
                    <button
                      disabled
                      class="rounded bg-gray-700 px-3 py-1 text-xs text-gray-400 cursor-not-allowed"
                    >
                      Installing…
                    </button>
                  {:else if !plugin.installed}
                    <button
                      class="rounded bg-blue-600 px-3 py-1 text-xs text-white transition-colors hover:bg-blue-700 dark:bg-blue-500 dark:hover:bg-blue-400 disabled:cursor-not-allowed disabled:opacity-50"
                      disabled={installing !== null}
                      onclick={() => handleInstall(plugin.name, plugin.marketplace)}
                    >
                      Install
                    </button>
                  {:else if plugin.version && plugin.installedVersion && plugin.version !== plugin.installedVersion}
                    <span class="text-xs text-gray-500 dark:text-gray-400">
                      {plugin.installedVersion} → {plugin.version}
                    </span>
                    <button
                      class="rounded bg-emerald-600 px-3 py-1 text-xs text-white transition-colors hover:bg-emerald-700 dark:bg-emerald-500 dark:hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-50"
                      disabled={installing !== null}
                      onclick={() => handleInstall(plugin.name, plugin.marketplace)}
                    >
                      Upgrade
                    </button>
                  {:else}
                    <button
                      class="rounded border border-gray-300 px-3 py-1 text-xs text-gray-600 transition-colors hover:bg-gray-100 dark:border-gray-600 dark:text-gray-400 dark:hover:bg-gray-800 disabled:cursor-not-allowed disabled:opacity-50"
                      disabled={installing !== null}
                      onclick={() => handleInstall(plugin.name, plugin.marketplace)}
                    >
                      Re-install
                    </button>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}

  <!-- Install progress output -->
  {#if installing !== null || installOutput.length > 0}
    <div class="mt-4 rounded border border-gray-700 bg-gray-950 p-3">
      <p class="mb-1 text-xs font-medium text-gray-400">
        {installing ? `Installing ${installing}…` : "Install complete"}
      </p>
      {#if installOutput.length > 0}
        <div class="max-h-32 overflow-auto font-mono text-xs text-gray-300">
          {#each installOutput as line (line)}
            <div>{line}</div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if pluginsStore.error}
    <div class="mt-4 rounded border border-red-800 bg-red-950 px-4 py-2">
      <p class="text-xs text-red-400">{pluginsStore.error}</p>
    </div>
  {/if}
</div>

<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events.js";
  import { t } from "$lib/i18n";

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
    <label for="marketplace-select" class="mb-1 block text-xs font-medium" style="color: var(--text-muted)">
      {t("plugins.marketplaceLabel")}
    </label>
    {#if pluginsStore.marketplaces.length === 0}
      <p class="text-sm" style="color: var(--text-muted)">{t("plugins.noMarketplacesHint")}</p>
    {:else}
      <select
        id="marketplace-select"
        bind:value={selectedMarketplace}
        class="input-base"
      >
        <option value="">{t("plugins.selectMarketplacePlaceholder")}</option>
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
        <p class="text-sm" style="color: var(--text-muted)">{t("plugins.noPluginsInMarketplace")}</p>
      </div>
    {:else}
      <div class="flex-1 overflow-auto">
        <div class="space-y-3">
          {#each pluginsStore.availablePlugins as plugin (plugin.name)}
            <div class="card">
              <div class="flex items-start justify-between gap-4">
                <!-- Plugin info -->
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-2">
                    <span class="font-semibold" style="color: var(--text-primary)">{plugin.name}</span>
                    {#if plugin.version}
                      <span class="text-xs" style="color: var(--text-muted)">v{plugin.version}</span>
                    {/if}
                    {#if plugin.category}
                      <span class="badge badge-info">
                        {plugin.category}
                      </span>
                    {/if}
                    {#if plugin.installed && plugin.installedVersion}
                      <span class="badge badge-success">
                        ✓ v{plugin.installedVersion}
                      </span>
                    {/if}
                  </div>
                  {#if plugin.description}
                    <p class="mt-1 text-xs" style="color: var(--text-secondary)">{plugin.description}</p>
                  {/if}
                </div>

                <!-- Action -->
                <div class="flex flex-shrink-0 items-center gap-2">
                  {#if installing === plugin.name}
                    <button
                      disabled
                      class="btn-primary"
                    >
                      {t("plugins.installing")}
                    </button>
                  {:else if !plugin.installed}
                    <button
                      class="btn-primary"
                      disabled={installing !== null}
                      onclick={() => handleInstall(plugin.name, plugin.marketplace)}
                    >
                      {t("plugins.install")}
                    </button>
                  {:else if plugin.version && plugin.installedVersion && plugin.version !== plugin.installedVersion}
                    <span class="text-xs" style="color: var(--text-muted)">
                      {plugin.installedVersion} → {plugin.version}
                    </span>
                    <button
                      class="btn-success"
                      disabled={installing !== null}
                      onclick={() => handleInstall(plugin.name, plugin.marketplace)}
                    >
                      {t("plugins.upgrade")}
                    </button>
                  {:else}
                    <button
                      class="btn-secondary"
                      disabled={installing !== null}
                      onclick={() => handleInstall(plugin.name, plugin.marketplace)}
                    >
                      {t("plugins.reinstall")}
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
    <div class="code-block mt-4">
      <p class="mb-1 text-xs font-medium">
        {installing ? t("plugins.installingName", { name: installing }) : t("plugins.installComplete")}
      </p>
      {#if installOutput.length > 0}
        <div class="max-h-32 overflow-auto">
          {#each installOutput as line (line)}
            <div>{line}</div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if pluginsStore.error}
    <div class="alert-error mt-4">
      {pluginsStore.error}
    </div>
  {/if}
</div>

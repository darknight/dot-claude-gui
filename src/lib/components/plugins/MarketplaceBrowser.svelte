<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events.js";
  import { t } from "$lib/i18n";
  import type { AvailablePlugin } from "$lib/api/types";

  let selectedMarketplace = $state<string>("");
  let installing = $state<string | null>(null); // plugin name being installed
  let installOutput = $state<string[]>([]);

  // Grouped: installed first (alpha), then available-to-install (alpha).
  const groupedPlugins = $derived.by(() => {
    const list = [...pluginsStore.availablePlugins].sort((a, b) =>
      a.name.localeCompare(b.name),
    );
    return {
      installed: list.filter((p) => p.installed),
      available: list.filter((p) => !p.installed),
    };
  });

  $effect(() => {
    pluginsStore.loadMarketplaces();
  });

  // Auto-select the first marketplace once the list is available, and drop a
  // stale selection if it's no longer in the list (e.g. marketplace removed
  // from the "Manage" tab).
  $effect(() => {
    const ids = pluginsStore.marketplaces.map((m) => m.id);
    if (ids.length === 0) {
      if (selectedMarketplace !== "") selectedMarketplace = "";
      return;
    }
    if (!ids.includes(selectedMarketplace)) {
      selectedMarketplace = ids[0];
    }
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

<div class="flex flex-1 overflow-hidden">
  <!-- Sidebar: marketplaces -->
  <aside
    class="w-64 flex-shrink-0 flex flex-col overflow-hidden"
    style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)"
  >
    <div
      class="px-4 py-3"
      style="border-bottom: 1px solid var(--border-color)"
    >
      <h2
        class="truncate text-xs font-semibold uppercase tracking-wider"
        style="color: var(--text-muted)"
      >
        {t("plugins.marketplace")}
        <span class="normal-case">({pluginsStore.marketplaces.length})</span>
      </h2>
    </div>

    <ul class="flex-1 overflow-y-auto py-1">
      {#if pluginsStore.marketplaces.length === 0}
        <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">
          {t("plugins.noMarketplacesHint")}
        </li>
      {:else}
        {#each pluginsStore.marketplaces as mp (mp.id)}
          <li>
            {#if selectedMarketplace === mp.id}
              <button
                type="button"
                class="flex w-full flex-col gap-0.5 px-4 py-2 text-left transition-colors"
                style="background-color: var(--accent-bg); color: var(--text-primary)"
                onclick={() => (selectedMarketplace = mp.id)}
              >
                <span class="truncate text-sm font-medium">{mp.id}</span>
                <span class="truncate font-mono text-xs" style="color: var(--text-muted)">{mp.repo}</span>
              </button>
            {:else}
              <button
                type="button"
                class="flex w-full flex-col gap-0.5 px-4 py-2 text-left transition-colors hover:bg-[var(--bg-card-hover)]"
                style="color: var(--text-secondary)"
                onclick={() => (selectedMarketplace = mp.id)}
              >
                <span class="truncate text-sm">{mp.id}</span>
                <span class="truncate font-mono text-xs" style="color: var(--text-muted)">{mp.repo}</span>
              </button>
            {/if}
          </li>
        {/each}
      {/if}
    </ul>
  </aside>

  <!-- Main: plugins in selected marketplace -->
  <main class="flex flex-1 flex-col overflow-hidden">
    <div class="flex-1 overflow-auto p-6">
      {#if !selectedMarketplace}
        <div class="flex h-full items-center justify-center">
          <p class="text-sm" style="color: var(--text-muted)">
            {pluginsStore.marketplaces.length === 0
              ? t("plugins.noMarketplacesHint")
              : t("plugins.selectMarketplaceHint")}
          </p>
        </div>
      {:else if pluginsStore.availablePlugins.length === 0}
        <div class="flex h-full items-center justify-center">
          <p class="text-sm" style="color: var(--text-muted)">{t("plugins.noPluginsInMarketplace")}</p>
        </div>
      {:else}
        {#snippet pluginCard(plugin: AvailablePlugin)}
          <div class="card">
            <div class="flex items-start justify-between gap-4">
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-2">
                  <span class="font-semibold" style="color: var(--text-primary)">{plugin.name}</span>
                  {#if plugin.version}
                    <span class="text-xs" style="color: var(--text-muted)">v{plugin.version}</span>
                  {/if}
                  {#if plugin.category}
                    <span class="badge badge-info">{plugin.category}</span>
                  {/if}
                  {#if plugin.installed && plugin.installedVersion}
                    <span class="badge badge-success">✓ v{plugin.installedVersion}</span>
                  {/if}
                </div>
                {#if plugin.description}
                  <p class="mt-1 text-xs" style="color: var(--text-secondary)">{plugin.description}</p>
                {/if}
              </div>

              <div class="flex flex-shrink-0 items-center gap-2">
                {#if installing === plugin.name}
                  <button disabled class="btn-primary">{t("plugins.installing")}</button>
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
        {/snippet}

        {#snippet groupHeader(label: string, count: number)}
          <h3
            class="px-1 pb-1 pt-2 text-xs font-semibold uppercase tracking-wider"
            style="color: var(--text-muted)"
          >
            {label} <span class="normal-case">({count})</span>
          </h3>
        {/snippet}

        {#if groupedPlugins.installed.length > 0}
          {@render groupHeader(t("plugins.installed"), groupedPlugins.installed.length)}
          <div class="mb-4 space-y-3">
            {#each groupedPlugins.installed as plugin (plugin.name)}
              {@render pluginCard(plugin)}
            {/each}
          </div>
        {/if}

        {#if groupedPlugins.available.length > 0}
          {@render groupHeader(t("plugins.available"), groupedPlugins.available.length)}
          <div class="space-y-3">
            {#each groupedPlugins.available as plugin (plugin.name)}
              {@render pluginCard(plugin)}
            {/each}
          </div>
        {/if}
      {/if}

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
        <div class="alert-error mt-4">{pluginsStore.error}</div>
      {/if}
    </div>
  </main>
</div>

<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events.js";
  import { t } from "$lib/i18n";

  let newRepo = $state("");
  let adding = $state(false);
  let addOutput = $state<string[]>([]);
  let removingId = $state<string | null>(null);
  let removeOutput = $state<string[]>([]);

  $effect(() => {
    pluginsStore.loadMarketplaces();
  });

  async function handleAdd() {
    const repo = newRepo.trim();
    if (!repo) return;
    adding = true;
    addOutput = [];

    const result = await pluginsStore.addMarketplace(repo);
    if (!result) {
      adding = false;
      return;
    }

    // Listen for output via Tauri IPC events
    const unsubOutput = await onCommandOutput((p) => {
      if (p.commandId === result.requestId) {
        addOutput = [...addOutput, p.line];
      }
    });
    const unsubCompleted = await onCommandCompleted((p) => {
      if (p.commandId === result.requestId) {
        adding = false;
        newRepo = "";
        pluginsStore.loadMarketplaces();
        unsubOutput();
        unsubCompleted();
      }
    });
  }

  async function handleRemove(id: string) {
    if (!confirm(t("plugins.confirmRemoveMarketplace", { id }))) return;
    removingId = id;
    removeOutput = [];

    const result = await pluginsStore.removeMarketplace(id);
    if (!result) {
      removingId = null;
      return;
    }

    // Listen for output via Tauri IPC events
    const unsubOutput = await onCommandOutput((p) => {
      if (p.commandId === result.requestId) {
        removeOutput = [...removeOutput, p.line];
      }
    });
    const unsubCompleted = await onCommandCompleted((p) => {
      if (p.commandId === result.requestId) {
        removingId = null;
        pluginsStore.loadMarketplaces();
        unsubOutput();
        unsubCompleted();
      }
    });
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden p-6">
  <!-- Add marketplace form -->
  <div class="card mb-6 p-4">
    <h3 class="mb-3 text-sm font-semibold" style="color: var(--text-primary)">{t("plugins.addMarketplace")}</h3>
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={newRepo}
        placeholder="owner/repo"
        class="input-base flex-1"
        onkeydown={(e) => { if (e.key === "Enter") handleAdd(); }}
        disabled={adding}
      />
      <button
        class="btn-primary px-4 py-1.5 text-sm"
        disabled={adding || !newRepo.trim()}
        onclick={handleAdd}
      >
        {adding ? t("plugins.adding") : t("common.add")}
      </button>
    </div>

    <!-- Add progress -->
    {#if adding || addOutput.length > 0}
      <div class="code-block mt-3">
        <p class="mb-1 text-xs font-medium" style="color: var(--text-muted)">
          {adding ? t("plugins.addingMarketplace") : t("plugins.done")}
        </p>
        {#if addOutput.length > 0}
          <div class="max-h-24 overflow-auto" style="color: var(--text-secondary)">
            {#each addOutput as line (line)}
              <div>{line}</div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Marketplace list -->
  {#if pluginsStore.marketplaces.length === 0}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">{t("plugins.noMarketplacesYet")}</p>
    </div>
  {:else}
    <div class="flex-1 overflow-auto">
      <div class="space-y-3">
        {#each pluginsStore.marketplaces as mp (mp.id)}
          <div class="card group relative">
            <div class="flex items-start justify-between gap-4">
              <!-- Marketplace info -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-semibold" style="color: var(--text-primary)">{mp.id}</span>
                </div>
                <div class="mt-0.5 flex flex-wrap items-center gap-2 text-xs" style="color: var(--text-muted)">
                  <span>{mp.repo}</span>
                  {#if mp.pluginCount !== undefined}
                    <span>·</span>
                    <span>{t("plugins.pluginCount", { count: mp.pluginCount })}</span>
                  {/if}
                  {#if mp.lastUpdated}
                    <span>·</span>
                    <span>{t("plugins.lastUpdated", { date: mp.lastUpdated })}</span>
                  {/if}
                </div>
                {#if mp.description}
                  <p class="mt-1 text-xs" style="color: var(--text-secondary)">{mp.description}</p>
                {/if}
              </div>

              <!-- Remove button -->
              <div class="flex-shrink-0">
                {#if removingId === mp.id}
                  <span class="text-xs" style="color: var(--text-muted)">{t("plugins.removing")}</span>
                {:else}
                  <button
                    class="btn-danger-ghost hidden group-hover:block disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={removingId !== null}
                    onclick={() => handleRemove(mp.id)}
                    title={t("plugins.removeMarketplaceTitle")}
                  >
                    {t("plugins.removeMarketplace")}
                  </button>
                {/if}
              </div>
            </div>

            <!-- Remove progress -->
            {#if removingId === mp.id && removeOutput.length > 0}
              <div class="code-block mt-2" style="color: var(--text-secondary)">
                {#each removeOutput as line (line)}
                  <div>{line}</div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if pluginsStore.error}
    <div class="alert-error mt-4">
      {pluginsStore.error}
    </div>
  {/if}
</div>

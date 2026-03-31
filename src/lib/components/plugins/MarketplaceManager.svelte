<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import type { WsEvent } from "$lib/api/types";

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

    const unsub = connectionStore.wsClient?.onEvent((event: WsEvent) => {
      if (event.type === "commandOutput" && event.commandId === result.requestId) {
        addOutput = [...addOutput, event.line];
      }
      if (event.type === "commandCompleted" && event.commandId === result.requestId) {
        adding = false;
        newRepo = "";
        pluginsStore.loadMarketplaces();
        unsub?.();
      }
    });
  }

  async function handleRemove(id: string) {
    if (!confirm(`Remove marketplace "${id}"?`)) return;
    removingId = id;
    removeOutput = [];

    const result = await pluginsStore.removeMarketplace(id);
    if (!result) {
      removingId = null;
      return;
    }

    const unsub = connectionStore.wsClient?.onEvent((event: WsEvent) => {
      if (event.type === "commandOutput" && event.commandId === result.requestId) {
        removeOutput = [...removeOutput, event.line];
      }
      if (event.type === "commandCompleted" && event.commandId === result.requestId) {
        removingId = null;
        pluginsStore.loadMarketplaces();
        unsub?.();
      }
    });
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden p-6">
  <!-- Add marketplace form -->
  <div class="mb-6 rounded-lg border border-gray-800 bg-gray-900 p-4">
    <h3 class="mb-3 text-sm font-semibold text-gray-200">Add Marketplace</h3>
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={newRepo}
        placeholder="owner/repo"
        class="flex-1 rounded border border-gray-700 bg-gray-800 px-3 py-1.5 text-sm text-gray-100 placeholder-gray-600 focus:border-blue-500 focus:outline-none"
        onkeydown={(e) => { if (e.key === "Enter") handleAdd(); }}
        disabled={adding}
      />
      <button
        class="rounded bg-blue-700 px-4 py-1.5 text-sm text-white transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
        disabled={adding || !newRepo.trim()}
        onclick={handleAdd}
      >
        {adding ? "Adding…" : "Add"}
      </button>
    </div>

    <!-- Add progress -->
    {#if adding || addOutput.length > 0}
      <div class="mt-3 rounded border border-gray-700 bg-gray-950 p-2">
        <p class="mb-1 text-xs font-medium text-gray-400">
          {adding ? "Adding marketplace…" : "Done"}
        </p>
        {#if addOutput.length > 0}
          <div class="max-h-24 overflow-auto font-mono text-xs text-gray-300">
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
      <p class="text-sm text-gray-600">No marketplaces registered yet.</p>
    </div>
  {:else}
    <div class="flex-1 overflow-auto">
      <div class="space-y-3">
        {#each pluginsStore.marketplaces as mp (mp.id)}
          <div class="group relative rounded-lg border border-gray-800 bg-gray-900 px-4 py-3 transition-colors hover:border-gray-700">
            <div class="flex items-start justify-between gap-4">
              <!-- Marketplace info -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-semibold text-gray-100">{mp.id}</span>
                </div>
                <div class="mt-0.5 flex flex-wrap items-center gap-2 text-xs text-gray-500">
                  <span>{mp.repo}</span>
                  {#if mp.pluginCount !== undefined}
                    <span>·</span>
                    <span>{mp.pluginCount} plugin{mp.pluginCount === 1 ? "" : "s"}</span>
                  {/if}
                  {#if mp.lastUpdated}
                    <span>·</span>
                    <span>updated {mp.lastUpdated}</span>
                  {/if}
                </div>
                {#if mp.description}
                  <p class="mt-1 text-xs text-gray-400">{mp.description}</p>
                {/if}
              </div>

              <!-- Remove button -->
              <div class="flex-shrink-0">
                {#if removingId === mp.id}
                  <span class="text-xs text-gray-500">Removing…</span>
                {:else}
                  <button
                    class="hidden rounded px-2 py-1 text-xs text-gray-500 transition-colors hover:bg-red-900/50 hover:text-red-400 group-hover:block disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={removingId !== null}
                    onclick={() => handleRemove(mp.id)}
                    title="Remove marketplace"
                  >
                    Remove
                  </button>
                {/if}
              </div>
            </div>

            <!-- Remove progress -->
            {#if removingId === mp.id && removeOutput.length > 0}
              <div class="mt-2 rounded border border-gray-700 bg-gray-950 p-2 font-mono text-xs text-gray-300">
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
    <div class="mt-4 rounded border border-red-800 bg-red-950 px-4 py-2">
      <p class="text-xs text-red-400">{pluginsStore.error}</p>
    </div>
  {/if}
</div>

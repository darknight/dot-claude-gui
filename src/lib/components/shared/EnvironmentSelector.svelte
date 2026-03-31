<script lang="ts">
  import { connectionsStore } from "$lib/stores/connections.svelte.js";
  import { connectionStore } from "$lib/stores/connection.svelte.js";
  import type { ConnectionEntry } from "$lib/api/types.js";

  let open = $state(false);

  function statusColor(entry: ConnectionEntry): string {
    if (entry.id === connectionsStore.activeConnectionId) {
      if (connectionStore.status === "connected") return "bg-green-400";
      if (connectionStore.status === "connecting") return "bg-yellow-400 animate-pulse";
      return "bg-red-400";
    }
    return "bg-gray-500";
  }

  async function selectConnection(entry: ConnectionEntry) {
    open = false;
    if (entry.id === connectionsStore.activeConnectionId) return;
    await connectionStore.switchConnection(entry);
  }

  function handleManage() {
    open = false;
    window.dispatchEvent(new CustomEvent("navigate", { detail: { nav: "A", sub: "connections" } }));
  }
</script>

<div class="relative">
  <button
    class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-gray-800 text-sm"
    onclick={() => (open = !open)}
  >
    <span class="w-2 h-2 rounded-full {statusColor(connectionsStore.activeConnection!)}"></span>
    <span class="text-gray-200">{connectionsStore.activeConnection?.name ?? "Local"}</span>
    <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)} onkeydown={() => {}}></div>

    <div class="absolute top-full left-0 mt-1 w-56 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50 py-1">
      {#each connectionsStore.connections as entry}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-700
            {entry.id === connectionsStore.activeConnectionId ? 'text-blue-400' : 'text-gray-300'}"
          onclick={() => selectConnection(entry)}
        >
          <span class="w-2 h-2 rounded-full {statusColor(entry)}"></span>
          <span class="flex-1">{entry.name}</span>
          {#if entry.managed}
            <span class="text-xs text-gray-500">自动</span>
          {/if}
        </button>
      {/each}

      <div class="border-t border-gray-700 my-1"></div>

      <button
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left text-gray-400 hover:bg-gray-700"
        onclick={handleManage}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        <span>管理连接...</span>
      </button>
    </div>
  {/if}
</div>

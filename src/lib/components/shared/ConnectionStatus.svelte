<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";
</script>

<div class="flex items-center gap-2 text-xs">
  <!-- Status dot -->
  {#if connectionStore.status === "connected"}
    <span class="h-2 w-2 rounded-full bg-green-500"></span>
    <span class="text-gray-400">v{connectionStore.daemonVersion}</span>
  {:else if connectionStore.status === "connecting"}
    <span class="h-2 w-2 animate-pulse rounded-full bg-yellow-400"></span>
    <span class="text-gray-400">Connecting...</span>
  {:else}
    <span class="h-2 w-2 rounded-full bg-red-500"></span>
    <span class="text-gray-400">Disconnected</span>
  {/if}

  {#if connectionStore.error}
    <span class="ml-1 text-red-400" title={connectionStore.error}>⚠</span>
  {/if}
</div>

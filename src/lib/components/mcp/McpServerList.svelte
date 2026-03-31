<script lang="ts">
  import { mcpStore } from "$lib/stores/mcp.svelte";

  // Group servers by scope
  const groupedServers = $derived(() => {
    const groups: Record<string, typeof mcpStore.servers> = {};
    for (const server of mcpStore.servers) {
      const scope = server.scope || "user";
      if (!groups[scope]) groups[scope] = [];
      groups[scope].push(server);
    }
    return groups;
  });

  const scopeOrder = ["user", "project", "local"];

  function transportBadgeClass(transport: string): string {
    if (transport === "stdio") return "bg-blue-900 text-blue-300";
    if (transport === "http") return "bg-green-900 text-green-300";
    if (transport === "sse") return "bg-yellow-900 text-yellow-300";
    return "bg-gray-800 text-gray-400";
  }

  function statusIndicatorClass(status?: string): string {
    if (!status) return "bg-gray-500";
    const s = status.toLowerCase();
    if (s === "connected") return "bg-green-500";
    if (s.includes("auth") || s.includes("needs")) return "bg-yellow-500";
    if (s === "error" || s.includes("fail")) return "bg-red-500";
    return "bg-gray-500";
  }

  function statusLabel(status?: string): string {
    if (!status) return "Unknown";
    return status;
  }

  async function handleRemove(name: string, scope: string) {
    await mcpStore.removeServer(name, scope);
    await mcpStore.loadServers();
  }
</script>

<div class="flex-1 overflow-auto p-6">
  {#if mcpStore.error}
    <div class="mb-4 rounded border border-red-800 bg-red-950 px-4 py-2">
      <p class="text-xs text-red-400">{mcpStore.error}</p>
    </div>
  {/if}

  {#if mcpStore.loading}
    <div class="flex items-center gap-2 py-4 text-sm text-gray-500">
      <span class="inline-block h-4 w-4 animate-spin rounded-full border-2 border-gray-600 border-t-blue-400"></span>
      Loading servers...
    </div>
  {:else if mcpStore.servers.length === 0}
    <div class="flex h-64 items-center justify-center">
      <p class="text-sm text-gray-600">No MCP servers configured</p>
    </div>
  {:else}
    <div class="space-y-6">
      {#each scopeOrder.filter(s => groupedServers()[s]?.length > 0) as scope}
        <div>
          <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">{scope}</h3>
          <div class="space-y-2">
            {#each groupedServers()[scope] as server (server.name + server.scope)}
              <div class="group relative rounded-lg border border-gray-800 bg-gray-900 px-4 py-3 transition-colors hover:border-gray-700">
                <div class="flex items-start justify-between gap-4">
                  <!-- Server info -->
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold text-gray-100">{server.name}</span>
                      <span class="rounded px-1.5 py-0.5 text-xs font-medium {transportBadgeClass(server.transport)}">
                        {server.transport}
                      </span>
                    </div>
                    <p class="mt-0.5 truncate text-xs text-gray-400 font-mono">
                      {server.transport === "stdio" ? (server.command ?? "") : (server.url ?? "")}
                      {#if server.transport === "stdio" && server.args?.length}
                        <span class="text-gray-600"> {server.args.join(" ")}</span>
                      {/if}
                    </p>
                  </div>

                  <!-- Status + remove -->
                  <div class="flex items-center gap-3">
                    <div class="flex items-center gap-1.5" title={statusLabel(server.status)}>
                      <span class="h-2 w-2 rounded-full {statusIndicatorClass(server.status)}"></span>
                      <span class="text-xs text-gray-500">{statusLabel(server.status)}</span>
                    </div>
                    <button
                      class="hidden rounded px-2 py-1 text-xs text-gray-500 transition-colors hover:bg-red-900/50 hover:text-red-400 group-hover:block"
                      onclick={() => handleRemove(server.name, server.scope)}
                      title="Remove server"
                    >
                      Remove
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

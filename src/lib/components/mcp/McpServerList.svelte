<script lang="ts">
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { t } from "$lib/i18n";

  // Group servers by origin (derived from name prefix / scope).
  // Claude Code's `claude mcp list` doesn't expose per-server scope, so
  // we group by heuristic: claude.ai hosted, plugin-provided, or local.
  function serverGroup(server: { name: string; scope: string }): string {
    if (server.name.startsWith("claude.ai ")) return "claude.ai";
    if (server.name.startsWith("plugin:")) return "plugin";
    if (server.scope === "user" || server.scope === "project" || server.scope === "local") {
      return server.scope;
    }
    return "other";
  }

  const groupedServers = $derived.by(() => {
    const groups: Record<string, typeof mcpStore.servers> = {};
    for (const server of mcpStore.servers) {
      const g = serverGroup(server);
      (groups[g] ??= []).push(server);
    }
    return groups;
  });

  const scopeOrder = ["claude.ai", "plugin", "user", "project", "local", "other"];

  function transportBadgeClass(transport: string): string {
    if (transport === "stdio") return "badge badge-info";
    if (transport === "http") return "badge badge-success";
    if (transport === "sse") return "badge badge-warning";
    return "badge badge-neutral";
  }

  function statusIndicatorStyle(status?: string): string {
    if (!status) return "background-color: var(--text-muted)";
    const s = status.toLowerCase();
    if (s === "connected") return "background-color: var(--status-success-text)";
    if (s.includes("auth") || s.includes("needs")) return "background-color: var(--status-warning-text)";
    if (s === "error" || s.includes("fail")) return "background-color: var(--status-error-text)";
    return "background-color: var(--text-muted)";
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
    <div class="alert-error mb-4">
      <p class="text-xs">{mcpStore.error}</p>
    </div>
  {/if}

  {#if mcpStore.loading}
    <div class="flex items-center gap-2 py-4 text-sm" style="color: var(--text-muted)">
      <span class="inline-block h-4 w-4 animate-spin rounded-full border-2" style="border-color: var(--border-strong); border-top-color: var(--accent-primary)"></span>
      {t("common.loading")}
    </div>
  {:else if mcpStore.servers.length === 0}
    <div class="flex h-64 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">{t("mcp.noServers")}</p>
    </div>
  {:else}
    <div class="space-y-6">
      {#each scopeOrder.filter(s => groupedServers[s]?.length > 0) as scope}
        <div>
          <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">{scope}</h3>
          <div class="space-y-2">
            {#each groupedServers[scope] as server (server.name + server.scope)}
              <div class="card group relative px-4 py-3 transition-colors">
                <div class="flex items-start justify-between gap-4">
                  <!-- Server info -->
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold" style="color: var(--text-primary)">{server.name}</span>
                      <span class="{transportBadgeClass(server.transport)}">
                        {server.transport}
                      </span>
                    </div>
                    <p class="mt-0.5 truncate text-xs font-mono" style="color: var(--text-secondary)">
                      {server.transport === "stdio" ? (server.command ?? "") : (server.url ?? "")}
                      {#if server.transport === "stdio" && server.args?.length}
                        <span style="color: var(--text-muted)"> {server.args.join(" ")}</span>
                      {/if}
                    </p>
                  </div>

                  <!-- Status + remove -->
                  <div class="flex items-center gap-3">
                    <div class="flex items-center gap-1.5" title={statusLabel(server.status)}>
                      <span class="h-2 w-2 rounded-full" style="{statusIndicatorStyle(server.status)}"></span>
                      <span class="text-xs" style="color: var(--text-muted)">{statusLabel(server.status)}</span>
                    </div>
                    <button
                      class="btn-danger-ghost hidden text-xs transition-colors group-hover:block"
                      onclick={() => handleRemove(server.name, server.scope)}
                      title={t("mcp.removeServer")}
                    >
                      {t("common.remove")}
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

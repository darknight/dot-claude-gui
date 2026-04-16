<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events";
  import type { CommandOutputPayload, CommandCompletedPayload } from "$lib/ipc/events";

  import type { PluginInfo } from "$lib/api/types";

  let pendingId = $state<string | null>(null);
  let outputLines = $state<string[]>([]);
  let collapsed = $state<Record<string, boolean>>({});

  const groups = $derived.by(() => {
    const map = new Map<string, PluginInfo[]>();
    for (const p of pluginsStore.plugins) {
      const key = p.marketplace || "unknown";
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(p);
    }
    return [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
  });

  function toggleGroup(name: string) {
    collapsed = { ...collapsed, [name]: !collapsed[name] };
  }

  async function handleUninstall(id: string) {
    const result = await pluginsStore.uninstallPlugin(id);
    if (!result?.requestId) return;

    pendingId = id;
    outputLines = [];

    const unlistenOutput = await onCommandOutput((p: CommandOutputPayload) => {
      if (p.commandId === result.requestId) {
        outputLines = [...outputLines, p.line];
      }
    });

    const unlistenCompleted = await onCommandCompleted(async (p: CommandCompletedPayload) => {
      if (p.commandId !== result.requestId) return;
      unlistenOutput();
      unlistenCompleted();
      pendingId = null;
      outputLines = [];

      if (p.exitCode === 0) {
        toastStore.success("Plugin uninstalled");
      } else {
        toastStore.error("Uninstall failed (exit code " + p.exitCode + ")");
      }
      await pluginsStore.loadPlugins();
    });
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
    <div class="space-y-1">
      {#each groups as [marketplaceName, plugins], groupIndex (marketplaceName)}
        {@const isCollapsed = collapsed[marketplaceName] ?? false}
        <div class={groupIndex === 0 ? "" : "pt-3"}>
          <button
            type="button"
            class="mb-2 flex w-full items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-gray-500 hover:text-gray-300"
            onclick={() => toggleGroup(marketplaceName)}
          >
            <span class="inline-block w-3 text-center">{isCollapsed ? "▸" : "▾"}</span>
            <span class="truncate">{marketplaceName}</span>
            <span class="text-gray-600">({plugins.length})</span>
          </button>
          {#if !isCollapsed}
            <div class="space-y-3 pl-5">
              {#each plugins as plugin (plugin.id)}
                <div class="group relative rounded-lg border border-gray-800 bg-gray-900 px-4 py-3 transition-colors hover:border-gray-700">
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="font-semibold text-gray-100">{plugin.name}</span>
                        {#if plugin.blocked}
                          <span class="rounded bg-red-900 px-1.5 py-0.5 text-xs font-medium text-red-300">
                            Blocked
                          </span>
                        {/if}
                      </div>
                      <div class="mt-0.5 text-xs text-gray-500">
                        v{plugin.version}
                      </div>
                      {#if plugin.description}
                        <p class="mt-1 text-xs text-gray-400">{plugin.description}</p>
                      {/if}
                    </div>

                    <div class="flex items-center gap-3">
                      <button
                        class="rounded px-2 py-1 text-xs text-gray-500 opacity-0 transition-opacity hover:bg-red-900/50 hover:text-red-400 group-hover:opacity-100 disabled:opacity-50"
                        onclick={() => handleUninstall(plugin.id)}
                        disabled={pendingId !== null}
                        title="Uninstall plugin"
                      >
                        {pendingId === plugin.id ? "Uninstalling..." : "Uninstall"}
                      </button>

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
      {/each}
    </div>
  {/if}

  {#if pendingId !== null && outputLines.length > 0}
    <div class="mt-4 rounded border border-gray-800 bg-gray-950 p-3">
      <pre class="max-h-32 overflow-auto font-mono text-xs text-gray-400 leading-relaxed">{outputLines.join("\n")}</pre>
    </div>
  {/if}
</div>

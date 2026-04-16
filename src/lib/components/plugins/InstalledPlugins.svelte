<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events";
  import type { CommandOutputPayload, CommandCompletedPayload } from "$lib/ipc/events";

  import type { PluginInfo } from "$lib/api/types";
  import { t } from "$lib/i18n";

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
        toastStore.success(t("plugins.uninstallSuccess"));
      } else {
        toastStore.error(t("plugins.uninstallFailed", { exitCode: p.exitCode }));
      }
      await pluginsStore.loadPlugins();
    });
  }
</script>

<div class="flex-1 overflow-auto p-6">
  {#if pluginsStore.loading}
    <p class="text-sm" style="color: var(--text-muted)">{t("common.loading")}</p>
  {:else if pluginsStore.error}
    <div class="alert-error mb-4">
      {pluginsStore.error}
    </div>
  {/if}

  {#if pluginsStore.plugins.length === 0 && !pluginsStore.loading}
    <div class="flex h-full items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">{t("plugins.noPlugins")}</p>
    </div>
  {:else}
    <div class="space-y-1">
      {#each groups as [marketplaceName, plugins], groupIndex (marketplaceName)}
        {@const isCollapsed = collapsed[marketplaceName] ?? false}
        <div class={groupIndex === 0 ? "" : "pt-3"}>
          <button
            type="button"
            class="mb-2 flex w-full items-center gap-1.5 text-xs font-semibold uppercase tracking-wider hover:opacity-80"
            style="color: var(--text-muted)"
            onclick={() => toggleGroup(marketplaceName)}
          >
            <span class="inline-block w-3 text-center">{isCollapsed ? "▸" : "▾"}</span>
            <span class="truncate">{marketplaceName}</span>
            <span>({plugins.length})</span>
          </button>
          {#if !isCollapsed}
            <div class="space-y-3 pl-5">
              {#each plugins as plugin (plugin.id)}
                <div class="card group relative">
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="font-semibold" style="color: var(--text-primary)">{plugin.name}</span>
                        {#if plugin.blocked}
                          <span class="badge badge-error">
                            {t("plugins.blocked")}
                          </span>
                        {/if}
                      </div>
                      <div class="mt-0.5 text-xs" style="color: var(--text-muted)">
                        v{plugin.version}
                      </div>
                      {#if plugin.description}
                        <p class="mt-1 text-xs" style="color: var(--text-secondary)">{plugin.description}</p>
                      {/if}
                    </div>

                    <div class="flex items-center gap-3">
                      <button
                        class="btn-danger-ghost opacity-0 transition-opacity group-hover:opacity-100 disabled:opacity-50"
                        onclick={() => handleUninstall(plugin.id)}
                        disabled={pendingId !== null}
                        title={t("plugins.uninstall")}
                      >
                        {pendingId === plugin.id ? t("plugins.uninstalling") : t("plugins.uninstall")}
                      </button>

                      <button
                        class="toggle-track"
                        role="switch"
                        aria-checked={plugin.enabled}
                        aria-label="Toggle {plugin.name}"
                        onclick={() => pluginsStore.togglePlugin(plugin.id, !plugin.enabled)}
                      >
                        <span class="toggle-knob"></span>
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
    <pre class="code-block mt-4 max-h-32 overflow-auto">{outputLines.join("\n")}</pre>
  {/if}
</div>

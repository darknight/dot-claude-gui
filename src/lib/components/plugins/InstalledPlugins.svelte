<script lang="ts">
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { onCommandOutput, onCommandCompleted } from "$lib/ipc/events";
  import type { CommandOutputPayload, CommandCompletedPayload } from "$lib/ipc/events";

  import type { PluginInfo } from "$lib/api/types";
  import { t } from "$lib/i18n";
  import { tick } from "svelte";

  let pendingId = $state<string | null>(null);
  let outputLines = $state<string[]>([]);
  let collapsed = $state<Record<string, boolean>>({});
  // Locally mirrors pluginsStore.highlightedPluginId for the duration of the
  // flash; cleared by a timer so re-entering the facet later doesn't replay.
  let flashedPluginId = $state<string | null>(null);
  let flashTimer: ReturnType<typeof setTimeout> | null = null;

  const groups = $derived.by(() => {
    const map = new Map<string, PluginInfo[]>();
    for (const p of pluginsStore.plugins) {
      const key = p.marketplace || "unknown";
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(p);
    }
    return [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
  });

  // Force-expand the marketplace group that contains a highlighted plugin —
  // otherwise the target card isn't in the DOM and the scroll-into-view is a
  // no-op.
  const forceExpandGroup = $derived.by(() => {
    if (!pluginsStore.highlightedPluginId) return null;
    const target = pluginsStore.plugins.find(p => p.id === pluginsStore.highlightedPluginId);
    return target?.marketplace ?? null;
  });

  function isCollapsedFor(name: string): boolean {
    if (forceExpandGroup === name) return false;
    return collapsed[name] ?? false;
  }

  function toggleGroup(name: string) {
    collapsed = { ...collapsed, [name]: !collapsed[name] };
  }

  // React to a jump request from elsewhere (e.g. SkillPreview's "open owning
  // plugin"). Scroll the card into view, flash a ring for 1.5s, then clear.
  $effect(() => {
    const id = pluginsStore.highlightedPluginId;
    if (!id) return;

    (async () => {
      await tick();
      const el = document.querySelector(`[data-plugin-id="${CSS.escape(id)}"]`);
      el?.scrollIntoView({ behavior: "smooth", block: "center" });
      flashedPluginId = id;
      if (flashTimer !== null) clearTimeout(flashTimer);
      flashTimer = setTimeout(() => {
        flashedPluginId = null;
        flashTimer = null;
        if (pluginsStore.highlightedPluginId === id) {
          pluginsStore.highlightPlugin(null);
        }
      }, 1500);
    })();

    return () => {
      if (flashTimer !== null) {
        clearTimeout(flashTimer);
        flashTimer = null;
      }
    };
  });

  async function handleUninstall(id: string) {
    // Reset prior failure output so a new attempt starts clean.
    outputLines = [];

    const result = await pluginsStore.uninstallPlugin(id);
    if (!result?.requestId) {
      // IPC itself failed (e.g. `claude` CLI not on PATH). The store has
      // already populated pluginsStore.error; surface it as a toast so the
      // user actually sees it instead of just the small alert at the top.
      toastStore.error(
        t("plugins.uninstallError", { msg: pluginsStore.error || "unknown" }),
      );
      return;
    }

    pendingId = id;

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

      if (p.exitCode === 0) {
        toastStore.success(t("plugins.uninstallSuccess"));
        outputLines = [];
      } else {
        toastStore.error(t("plugins.uninstallFailed", { exitCode: p.exitCode }));
        // Retain outputLines on failure so the user can see why it failed
        // (the toast only shows the exit code). Cleared on next attempt.
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
        {@const isCollapsed = isCollapsedFor(marketplaceName)}
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
              {#each plugins as plugin (plugin.id + ":" + plugin.scope + ":" + (plugin.projectPath ?? ""))}
                {@const isProjectScope = plugin.scope === "project"}
                <div
                  data-plugin-id={plugin.id}
                  class="card group relative {flashedPluginId === plugin.id ? 'plugin-flash' : ''}"
                >
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="font-semibold" style="color: var(--text-primary)">{plugin.name}</span>
                        {#if plugin.blocked}
                          <span class="badge badge-error">
                            {t("plugins.blocked")}
                          </span>
                        {/if}
                        {#if isProjectScope}
                          <span class="badge badge-info" title={plugin.projectPath ?? ""}>
                            {t("plugins.scopeProject")}
                          </span>
                        {/if}
                      </div>
                      <div class="mt-0.5 text-xs" style="color: var(--text-muted)">
                        v{plugin.version}
                      </div>
                      {#if plugin.description}
                        <p class="mt-1 text-xs" style="color: var(--text-secondary)">{plugin.description}</p>
                      {/if}
                      {#if isProjectScope && plugin.projectPath}
                        <p class="mt-1 truncate text-xs" style="color: var(--text-muted)">
                          {t("plugins.installedFromProject", { path: plugin.projectPath })}
                        </p>
                      {/if}
                    </div>

                    <div class="flex items-center gap-3">
                      <button
                        class="btn-danger-ghost opacity-0 transition-opacity group-hover:opacity-100 disabled:opacity-50"
                        onclick={() => handleUninstall(plugin.id)}
                        disabled={pendingId !== null || isProjectScope}
                        title={isProjectScope ? t("plugins.uninstallProjectHint") : t("plugins.uninstall")}
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

  {#if outputLines.length > 0}
    <pre class="code-block mt-4 max-h-32 overflow-auto">{outputLines.join("\n")}</pre>
  {/if}
</div>

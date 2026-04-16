<script lang="ts">
  import { ipcClient } from "$lib/ipc/client.js";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import type { EffectiveConfig } from "$lib/api/types";

  let effective = $state<EffectiveConfig | null>(null);
  let loading = $state(false);
  let error = $state("");
  let expandedSections = $state<Record<string, boolean>>({});
  let rawExpanded = $state(false);

  const activeProject = $derived(projectsStore.activeProject);

  $effect(() => {
    if (activeProject) {
      loadEffective(activeProject.id);
    } else {
      effective = null;
    }
  });

  async function loadEffective(projectId: string) {
    loading = true;
    error = "";
    try {
      effective = await ipcClient.getEffectiveConfig(projectId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load";
    } finally {
      loading = false;
    }
  }

  function getSource(key: string): string | null {
    return effective?.fieldSources[key] ?? null;
  }

  const sourceBadgeClass: Record<string, string> = {
    user: "badge badge-info",
    project: "badge badge-success",
    local: "badge badge-warning",
    managed: "badge badge-error",
  };

  const sections = [
    { key: "permissions", label: "Permissions" },
    { key: "hooks", label: "Hooks" },
    { key: "env", label: "Environment" },
    { key: "sandbox", label: "Sandbox" },
    { key: "enabledPlugins", label: "Enabled Plugins" },
    { key: "statusLine", label: "Status Line" },
    { key: "deniedMcpServers", label: "Denied MCP Servers" },
  ];

  function toggleSection(key: string) {
    expandedSections = { ...expandedSections, [key]: !expandedSections[key] };
  }

  function getSectionValue(key: string): unknown {
    return effective?.settings?.[key] ?? null;
  }

  function formatJson(value: unknown): string {
    return JSON.stringify(value, null, 2);
  }

  function hasValue(value: unknown): boolean {
    if (value === null || value === undefined) return false;
    if (Array.isArray(value)) return value.length > 0;
    if (typeof value === "object") return Object.keys(value as object).length > 0;
    return true;
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !activeProject}
    <!-- No project selected -->
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">
        Select a project from the header dropdown to view effective config
      </p>
    </div>

  {:else if loading}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">Loading effective config...</p>
    </div>

  {:else if error}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--status-error-text)">{error}</p>
    </div>

  {:else if effective}
    <div class="flex-1 overflow-y-auto p-6 space-y-3">

      <!-- Section heading -->
      <div class="mb-4">
        <h2 class="text-sm font-semibold" style="color: var(--text-primary)">
          Effective Config — {activeProject.name}
        </h2>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">
          Merged settings with per-field source annotations.
        </p>
      </div>

      <!-- Config sections -->
      {#each sections as section}
        {@const value = getSectionValue(section.key)}
        {@const source = getSource(section.key)}
        {@const isExpanded = expandedSections[section.key] ?? false}

        <div class="card overflow-hidden" style="padding: 0;">
          <!-- Section header (always visible, clickable) -->
          <button
            class="flex w-full items-center justify-between px-4 py-3 text-left hover:bg-[var(--bg-card-hover)] transition-colors"
            onclick={() => toggleSection(section.key)}
          >
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium" style="color: var(--text-primary)">{section.label}</span>
              {#if source}
                <span class="capitalize {sourceBadgeClass[source] ?? 'badge badge-neutral'}">
                  {source}
                </span>
              {/if}
              {#if !hasValue(value)}
                <span class="text-xs" style="color: var(--text-muted)">(not set)</span>
              {/if}
            </div>
            <span class="text-xs" style="color: var(--text-muted)">{isExpanded ? "▲" : "▼"}</span>
          </button>

          <!-- Section content (collapsible) -->
          {#if isExpanded}
            <div class="border-t px-4 py-3" style="border-color: var(--border-color)">
              {#if hasValue(value)}
                <pre class="code-block overflow-x-auto leading-relaxed">{formatJson(value)}</pre>
              {:else}
                <p class="text-xs italic" style="color: var(--text-muted)">No value configured for this section.</p>
              {/if}
            </div>
          {/if}
        </div>
      {/each}

      <!-- Raw effective JSON -->
      <div class="card overflow-hidden" style="padding: 0;">
        <button
          class="flex w-full items-center justify-between px-4 py-3 text-left hover:bg-[var(--bg-card-hover)] transition-colors"
          onclick={() => { rawExpanded = !rawExpanded; }}
        >
          <span class="text-sm font-medium" style="color: var(--text-primary)">Raw Effective JSON</span>
          <span class="text-xs" style="color: var(--text-muted)">{rawExpanded ? "▲" : "▼"}</span>
        </button>
        {#if rawExpanded}
          <div class="border-t px-4 py-3" style="border-color: var(--border-color)">
            <pre class="code-block overflow-x-auto leading-relaxed">{formatJson(effective.settings)}</pre>
          </div>
        {/if}
      </div>

    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">No effective config available.</p>
    </div>
  {/if}
</div>

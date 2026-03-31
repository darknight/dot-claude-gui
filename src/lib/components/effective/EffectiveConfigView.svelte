<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";
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
    const client = connectionStore.client;
    if (!client) return;
    loading = true;
    error = "";
    try {
      effective = await client.getEffectiveConfig(projectId);
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
    user: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
    project: "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
    local: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300",
    managed: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300",
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
      <p class="text-sm text-gray-500">
        Select a project from the header dropdown to view effective config
      </p>
    </div>

  {:else if loading}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-500">Loading effective config...</p>
    </div>

  {:else if error}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-red-400">{error}</p>
    </div>

  {:else if effective}
    <div class="flex-1 overflow-y-auto p-6 space-y-3">

      <!-- Section heading -->
      <div class="mb-4">
        <h2 class="text-sm font-semibold text-gray-200">
          Effective Config — {activeProject.name}
        </h2>
        <p class="mt-1 text-xs text-gray-500">
          Merged settings with per-field source annotations.
        </p>
      </div>

      <!-- Config sections -->
      {#each sections as section}
        {@const value = getSectionValue(section.key)}
        {@const source = getSource(section.key)}
        {@const isExpanded = expandedSections[section.key] ?? false}

        <div class="rounded-lg border border-gray-700 bg-gray-900 overflow-hidden">
          <!-- Section header (always visible, clickable) -->
          <button
            class="flex w-full items-center justify-between px-4 py-3 text-left hover:bg-gray-800/50 transition-colors"
            onclick={() => toggleSection(section.key)}
          >
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-gray-200">{section.label}</span>
              {#if source}
                <span
                  class="rounded px-1.5 py-0.5 text-xs font-medium capitalize {sourceBadgeClass[source] ?? 'bg-gray-700 text-gray-300'}"
                >
                  {source}
                </span>
              {/if}
              {#if !hasValue(value)}
                <span class="text-xs text-gray-600">(not set)</span>
              {/if}
            </div>
            <span class="text-xs text-gray-500">{isExpanded ? "▲" : "▼"}</span>
          </button>

          <!-- Section content (collapsible) -->
          {#if isExpanded}
            <div class="border-t border-gray-700 px-4 py-3">
              {#if hasValue(value)}
                <pre class="overflow-x-auto rounded bg-gray-950 p-3 text-xs text-gray-300 leading-relaxed">{formatJson(value)}</pre>
              {:else}
                <p class="text-xs text-gray-600 italic">No value configured for this section.</p>
              {/if}
            </div>
          {/if}
        </div>
      {/each}

      <!-- Raw effective JSON -->
      <div class="rounded-lg border border-gray-700 bg-gray-900 overflow-hidden">
        <button
          class="flex w-full items-center justify-between px-4 py-3 text-left hover:bg-gray-800/50 transition-colors"
          onclick={() => { rawExpanded = !rawExpanded; }}
        >
          <span class="text-sm font-medium text-gray-200">Raw Effective JSON</span>
          <span class="text-xs text-gray-500">{rawExpanded ? "▲" : "▼"}</span>
        </button>
        {#if rawExpanded}
          <div class="border-t border-gray-700 px-4 py-3">
            <pre class="overflow-x-auto rounded bg-gray-950 p-3 text-xs text-gray-300 leading-relaxed">{formatJson(effective.settings)}</pre>
          </div>
        {/if}
      </div>

    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">No effective config available.</p>
    </div>
  {/if}
</div>

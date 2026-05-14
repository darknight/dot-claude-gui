<script lang="ts">
  import McpModule from "$lib/components/mcp/McpModule.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  const sections = [
    { id: "servers", labelKey: "mcp.servers" },
    { id: "add", labelKey: "mcp.addServer" },
  ] satisfies { id: string; labelKey: MessageKey }[];

  let active = $state("servers");
</script>

<div class="flex flex-col flex-1 overflow-hidden">
  <nav
    class="flex items-center gap-1 px-2 py-1"
    style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
  >
    {#each sections as section (section.id)}
      <button
        type="button"
        class="px-2.5 py-1 text-xs rounded transition-colors {active === section.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
        style="background-color: {active === section.id ? 'var(--accent-bg)' : 'transparent'}; color: {active === section.id ? 'var(--accent-text)' : 'var(--text-secondary)'}"
        onclick={() => { active = section.id; }}
      >
        {t(section.labelKey)}
      </button>
    {/each}
  </nav>

  <div class="flex-1 flex flex-col overflow-hidden">
    <McpModule activeSection={active} />
  </div>
</div>

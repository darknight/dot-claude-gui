<script lang="ts">
  import McpServerList from "./McpServerList.svelte";
  import McpServerEditor from "./McpServerEditor.svelte";

  let { activeSection = "servers" }: { activeSection: string } = $props();
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if activeSection === "servers"}
    <McpServerList />
  {:else if activeSection === "add"}
    <McpServerEditor />
  {:else}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">Unknown section: {activeSection}</p>
    </div>
  {/if}
</div>

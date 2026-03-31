<script lang="ts">
  import InstalledPlugins from "./InstalledPlugins.svelte";

  let { activeSection = "installed" }: { activeSection: string } = $props();
</script>

<!-- Error display is handled inside InstalledPlugins -->

<div class="flex flex-1 flex-col overflow-hidden">
  {#if activeSection === "installed"}
    <InstalledPlugins />
  {:else if activeSection === "marketplace"}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Marketplace browser — coming in Task 9</p>
    </div>
  {:else if activeSection === "manage-marketplaces"}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Manage Marketplaces — coming in Task 9</p>
    </div>
  {:else if activeSection === "per-project"}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Per-Project activation — coming in Task 10</p>
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Unknown section: {activeSection}</p>
    </div>
  {/if}
</div>

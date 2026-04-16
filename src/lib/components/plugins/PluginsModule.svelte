<script lang="ts">
  import InstalledPlugins from "./InstalledPlugins.svelte";
  import MarketplaceBrowser from "./MarketplaceBrowser.svelte";
  import MarketplaceManager from "./MarketplaceManager.svelte";
  import ProjectActivation from "./ProjectActivation.svelte";

  let { activeSection = "installed" }: { activeSection: string } = $props();
</script>

<!-- Error display is handled inside InstalledPlugins -->

<div class="flex flex-1 flex-col overflow-hidden">
  {#if activeSection === "installed"}
    <InstalledPlugins />
  {:else if activeSection === "marketplace"}
    <MarketplaceBrowser />
  {:else if activeSection === "manage-marketplaces"}
    <MarketplaceManager />
  {:else if activeSection === "per-project"}
    <ProjectActivation />
  {:else}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">Unknown section: {activeSection}</p>
    </div>
  {/if}
</div>

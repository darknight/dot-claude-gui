<script lang="ts">
  import { t, type MessageKey } from "$lib/i18n";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { modeStore, type ProjectFacetKey } from "$lib/stores/mode.svelte";
  import BindingFacet from "./BindingFacet.svelte";
  import LaunchFacet from "./LaunchFacet.svelte";
  import PluginsOverrideFacet from "./PluginsOverrideFacet.svelte";
  import ProjectSettingsFacet from "./ProjectSettingsFacet.svelte";
  import ProjectMemoryFacet from "./ProjectMemoryFacet.svelte";
  import ProjectClaudeMdFacet from "./ProjectClaudeMdFacet.svelte";
  import EffectiveFacet from "./EffectiveFacet.svelte";
  import StalePathBanner from "./StalePathBanner.svelte";

  const FACETS: Array<{ key: ProjectFacetKey; labelKey: MessageKey }> = [
    { key: "binding",   labelKey: "projectMode.facet.binding" },
    { key: "launch",    labelKey: "projectMode.facet.launch" },
    { key: "plugins",   labelKey: "projectMode.facet.plugins" },
    { key: "settings",  labelKey: "projectMode.facet.settings" },
    { key: "memory",    labelKey: "projectMode.facet.memory" },
    { key: "claudemd",  labelKey: "projectMode.facet.claudemd" },
    { key: "effective", labelKey: "projectMode.facet.effective" },
  ];

  const selected = $derived(projectsStore.currentBinding);
  const isStale = $derived(projectsStore.currentStale);
  const isBound = $derived(projectsStore.currentBound);
  const activeFacet = $derived(modeStore.projectFacet(modeStore.selectedProject));

  function tabDisabled(key: ProjectFacetKey): boolean {
    if (isStale) return true;          // stale: all disabled, banner only
    if (!isBound && key !== "binding") return true; // unbound: only Binding
    return false;
  }
</script>

{#if !selected}
  <div class="empty">{t("projectMode.selectProject")}</div>
{:else}
  <div class="project-mode">
    {#if isStale}
      <StalePathBanner path={selected.path} />
    {/if}
    <nav class="tabs" role="tablist">
      {#each FACETS as f (f.key)}
        <button
          role="tab"
          aria-selected={activeFacet === f.key}
          disabled={tabDisabled(f.key)}
          class:active={activeFacet === f.key}
          onclick={() => {
              if (modeStore.selectedProject) {
                modeStore.setProjectFacet(modeStore.selectedProject, f.key);
              }
            }}
        >{t(f.labelKey)}</button>
      {/each}
    </nav>
    <div class="facet">
      {#if isStale}
        <div class="empty">{t("projectMode.stalePathBlocked")}</div>
      {:else if activeFacet === "binding"}
        <BindingFacet path={selected.path} />
      {:else if activeFacet === "launch"}
        <LaunchFacet path={selected.path} />
      {:else if activeFacet === "plugins"}
        <PluginsOverrideFacet path={selected.path} />
      {:else if activeFacet === "settings"}
        <ProjectSettingsFacet path={selected.path} />
      {:else if activeFacet === "memory"}
        <ProjectMemoryFacet path={selected.path} />
      {:else if activeFacet === "claudemd"}
        <ProjectClaudeMdFacet path={selected.path} />
      {:else if activeFacet === "effective"}
        <EffectiveFacet path={selected.path} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .project-mode {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .tabs {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    background: var(--bg-secondary, transparent);
  }
  .tabs button {
    padding: 4px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--text-primary);
    cursor: pointer;
    white-space: nowrap;
  }
  .tabs button.active {
    background: var(--bg-tab-active, var(--bg-primary));
    border-color: var(--border);
  }
  .tabs button[disabled] {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .facet {
    flex: 1;
    overflow: auto;
  }
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
</style>

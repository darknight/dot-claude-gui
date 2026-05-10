<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { launcherStore } from "$lib/stores/launcher.svelte";
  import { t } from "$lib/i18n";
</script>

<div class="flex flex-col overflow-hidden h-full">
  <!-- Project selector -->
  <div class="px-4 py-3 border-b" style="border-color: var(--border-color)">
    {#if projectsStore.projects.length === 0}
      <span class="text-xs" style="color: var(--text-muted)">{t("launcher.noProjects")}</span>
    {:else}
      <select
        class="input-base w-full text-xs"
        style="padding: 0.375rem 0.5rem"
        value={launcherStore.selectedProjectId}
        onchange={(e) => {
          const val = (e.target as HTMLSelectElement).value;
          launcherStore.selectProject(val);
        }}
      >
        <option value="">{t("launcher.selectProjectPlaceholder")}</option>
        {#each projectsStore.projects as project (project.id)}
          <option value={project.id}>{project.name}</option>
        {/each}
      </select>
    {/if}
  </div>
</div>

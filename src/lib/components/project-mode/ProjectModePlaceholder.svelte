<script lang="ts">
  import { modeStore } from "$lib/stores/mode.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { t } from "$lib/i18n";

  const selected = $derived(
    projectsStore.entries.find((p) => p.path === modeStore.selectedProject) ?? null
  );
</script>

<div class="flex-1 flex items-center justify-center p-6">
  <div class="text-center">
    <p class="text-sm" style="color: var(--text-muted)">
      {#if selected}
        {t("shell.projectModeComing", { path: selected.path })}
      {:else}
        {t("shell.projectModeSelectHint")}
      {/if}
    </p>
  </div>
</div>

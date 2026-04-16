<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { projectsStore } from "$lib/stores/projects.svelte.js";
  import { configStore } from "$lib/stores/config.svelte.js";
  import { claudeMdStore } from "$lib/stores/claudemd.svelte.js";
  import { memoryStore } from "$lib/stores/memory.svelte.js";
  import { t } from "$lib/i18n";

  let open = $state(false);
  let addError = $state("");

  function selectUserScope() {
    open = false;
    projectsStore.selectProject(null);
    configStore.activeScope = "user";
    configStore.loadUserConfig();
  }

  function selectProject(id: string) {
    open = false;
    projectsStore.selectProject(id);
    configStore.activeScope = "project";
    configStore.loadProjectConfig(id);
  }

  async function addProject() {
    addError = "";
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: t("scope.selectProjectDir"),
      });
      if (selected) {
        await projectsStore.registerProject(selected);
        // Refresh per-project data that depends on the project registry
        await Promise.all([
          claudeMdStore.loadFiles(),
          memoryStore.loadProjects(),
        ]);
      }
    } catch (err) {
      addError = err instanceof Error ? err.message : String(err);
    }
  }

  const displayName = $derived(
    projectsStore.activeProjectId
      ? projectsStore.activeProject?.path.replace(/^.*\//, "") ?? t("scope.projectFallback")
      : t("scope.user")
  );

  const displayIcon = $derived(projectsStore.activeProjectId ? "📁" : "🏠");
</script>

<div class="relative">
  <button
    class="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm transition-colors"
    style="color: var(--text-primary)"
    onmouseenter={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-card-hover)'}
    onmouseleave={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = ''}
    onclick={() => (open = !open)}
  >
    <span>{displayIcon}</span>
    <span style="color: var(--text-primary)">{displayName}</span>
    <svg class="w-3 h-3" style="color: var(--text-muted)" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)} onkeydown={() => {}}></div>

    <div class="absolute top-full left-0 mt-1 w-80 rounded-lg shadow-xl z-50 py-1" style="background-color: var(--bg-card); border: 1px solid var(--border-strong)">
      <button
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left transition-colors"
        style="color: {!projectsStore.activeProjectId ? 'var(--accent-text)' : 'var(--text-secondary)'}"
        onmouseenter={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-tertiary)'}
        onmouseleave={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = ''}
        onclick={selectUserScope}
      >
        <span>🏠</span>
        <span>{t("scope.user")}</span>
      </button>

      {#if projectsStore.projects.length > 0}
        <div class="my-1" style="border-top: 1px solid var(--border-strong)"></div>
      {/if}

      {#each projectsStore.projects as project}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left transition-colors"
          style="color: {projectsStore.activeProjectId === project.id ? 'var(--accent-text)' : 'var(--text-secondary)'}"
          onmouseenter={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-tertiary)'}
          onmouseleave={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = ''}
          onclick={() => selectProject(project.id)}
          title={project.path}
        >
          <span>📁</span>
          <span class="flex-1 truncate">{project.path}</span>
        </button>
      {/each}

      <div class="my-1" style="border-top: 1px solid var(--border-strong)"></div>

      <button
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left transition-colors"
        style="color: var(--text-muted)"
        onmouseenter={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--bg-tertiary)'}
        onmouseleave={(e) => (e.currentTarget as HTMLElement).style.backgroundColor = ''}
        onclick={addProject}
      >
        <span>+</span>
        <span>{t("scope.addProject")}</span>
      </button>

      {#if addError}
        <div class="px-3 py-1">
          <p class="text-xs" style="color: var(--status-error-text)">{addError}</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

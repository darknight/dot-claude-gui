<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte.js";
  import { configStore } from "$lib/stores/config.svelte.js";

  let open = $state(false);
  let showAddProject = $state(false);
  let newProjectPath = $state("");
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
    if (!newProjectPath.trim()) return;
    addError = "";
    try {
      await projectsStore.registerProject(newProjectPath.trim());
      newProjectPath = "";
      showAddProject = false;
    } catch (err) {
      addError = err instanceof Error ? err.message : String(err);
    }
  }

  const displayName = $derived(
    projectsStore.activeProjectId
      ? projectsStore.activeProject?.path.replace(/^.*\//, "") ?? "Project"
      : "User Scope"
  );

  const displayIcon = $derived(projectsStore.activeProjectId ? "📁" : "🏠");
</script>

<div class="relative">
  <button
    class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-gray-800 text-sm"
    onclick={() => (open = !open)}
  >
    <span>{displayIcon}</span>
    <span class="text-gray-200">{displayName}</span>
    <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-40" onclick={() => (open = false)} onkeydown={() => {}}></div>

    <div class="absolute top-full left-0 mt-1 w-72 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50 py-1">
      <button
        class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-700
          {!projectsStore.activeProjectId ? 'text-blue-400' : 'text-gray-300'}"
        onclick={selectUserScope}
      >
        <span>🏠</span>
        <span>User Scope</span>
      </button>

      {#if projectsStore.projects.length > 0}
        <div class="border-t border-gray-700 my-1"></div>
      {/if}

      {#each projectsStore.projects as project}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-gray-700
            {projectsStore.activeProjectId === project.id ? 'text-blue-400' : 'text-gray-300'}"
          onclick={() => selectProject(project.id)}
        >
          <span>📁</span>
          <span class="flex-1 truncate">{project.path}</span>
        </button>
      {/each}

      <div class="border-t border-gray-700 my-1"></div>

      {#if showAddProject}
        <div class="px-3 py-2">
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newProjectPath}
              placeholder="/path/to/project"
              class="flex-1 bg-gray-900 border border-gray-600 rounded px-2 py-1 text-sm text-gray-200"
              onkeydown={(e) => e.key === "Enter" && addProject()}
            />
            <button
              class="px-2 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-500"
              onclick={addProject}
            >
              添加
            </button>
          </div>
          {#if addError}
            <p class="text-xs text-red-400 mt-1">{addError}</p>
          {/if}
        </div>
      {:else}
        <button
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left text-gray-400 hover:bg-gray-700"
          onclick={() => (showAddProject = true)}
        >
          <span>+</span>
          <span>添加项目...</span>
        </button>
      {/if}
    </div>
  {/if}
</div>

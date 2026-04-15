<script lang="ts">
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { t } from "$lib/i18n";

  // In project scope, the memory project is dictated by ScopeSelector.
  // Disable the manual dropdown so the UI is unambiguous.
  const isProjectScope = $derived(configStore.activeScope === "project");

  // Auto-sync with the active project in ScopeSelector:
  // when the active project has a memory dir, show its memory files.
  // If the active project has no memory, clear the selection so the user
  // sees an explicit "no memory" state instead of stale files from another project.
  $effect(() => {
    const activeProject = projectsStore.activeProject;
    if (!activeProject) return;
    const match = memoryStore.projects.find(
      (p) => p.projectPath === activeProject.path,
    );
    if (match) {
      if (memoryStore.activeProjectId !== match.id) {
        memoryStore.selectProject(match.id);
      }
    } else {
      // Clear selection — this project has no memory directory yet.
      if (memoryStore.activeProjectId !== null) {
        memoryStore.clearSelection();
      }
    }
  });

  // Whether the active scope project has no memory directory.
  const activeProjectHasNoMemory = $derived(
    projectsStore.activeProject != null &&
      !memoryStore.projects.some(
        (p) => p.projectPath === projectsStore.activeProject?.path,
      ),
  );

  // Auto-select MEMORY.md (or first file) once files are loaded.
  $effect(() => {
    if (memoryStore.files.length === 0) return;
    if (memoryStore.activeFile) return;
    const memoryMd = memoryStore.files.find((f) => f.filename === "MEMORY.md");
    const target = memoryMd ?? memoryStore.files[0];
    memoryStore.selectFile(target.filename);
  });

  function typeBadgeClass(memoryType?: string): string {
    switch (memoryType) {
      case "core":
        return "bg-blue-900 text-blue-300";
      case "project":
        return "bg-purple-900 text-purple-300";
      case "session":
        return "bg-yellow-900 text-yellow-300";
      default:
        return "bg-gray-700 text-gray-300";
    }
  }
</script>

<div class="flex flex-col overflow-hidden h-full">
  <!-- Project selector -->
  <div class="px-4 py-3 border-b border-gray-800">
    {#if memoryStore.projects.length === 0}
      <span class="text-xs text-gray-600">No memory projects</span>
    {:else}
      <select
        class="w-full rounded bg-gray-800 px-2 py-1.5 text-xs text-gray-300 focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
        value={memoryStore.activeProjectId ?? ""}
        disabled={isProjectScope}
        title={isProjectScope ? t("memory.dropdownDisabledInProjectScope") : ""}
        onchange={(e) => {
          const val = (e.target as HTMLSelectElement).value;
          if (val) memoryStore.selectProject(val);
        }}
      >
        <option value="">Select project...</option>
        {#each memoryStore.projects as project (project.id)}
          <option value={project.id}>{project.projectPath}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- File list -->
  <ul class="flex-1 overflow-y-auto py-2">
    {#if memoryStore.loading && memoryStore.activeProjectId && memoryStore.files.length === 0}
      <li class="px-4 py-2 text-xs text-gray-500">Loading...</li>
    {:else if activeProjectHasNoMemory}
      <li class="px-4 py-2 text-xs text-gray-500">
        {t("memory.noFilesYet", {
          name: projectsStore.activeProject?.path.replace(/^.*\//, "") ?? "",
        })}
      </li>
    {:else if !memoryStore.activeProjectId}
      <li class="px-4 py-2 text-xs text-gray-600">Select a project above</li>
    {:else if memoryStore.files.length === 0}
      <li class="px-4 py-2 text-xs text-gray-600">No memory files</li>
    {:else}
      {#each memoryStore.files as file (file.filename)}
        <li>
          <button
            class="flex w-full flex-col gap-0.5 px-4 py-2 text-left transition-colors
              {memoryStore.activeFile?.filename === file.filename
              ? 'bg-gray-800 text-white'
              : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
            onclick={() => memoryStore.selectFile(file.filename)}
          >
            <div class="flex items-center justify-between gap-2">
              <span class="truncate text-sm">{file.name ?? file.filename}</span>
              {#if file.memoryType}
                <span class="flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {typeBadgeClass(file.memoryType)}">
                  {file.memoryType}
                </span>
              {/if}
            </div>
            {#if file.name && file.filename !== file.name}
              <span class="truncate font-mono text-xs text-gray-600">{file.filename}</span>
            {/if}
          </button>
        </li>
      {/each}
    {/if}
  </ul>

  {#if memoryStore.error}
    <div class="px-4 py-2 text-xs text-red-400 border-t border-gray-800">
      {memoryStore.error}
    </div>
  {/if}
</div>

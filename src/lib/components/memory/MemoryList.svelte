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
      case "core": return "badge badge-info";
      case "project": return "badge badge-purple";
      case "session": return "badge badge-warning";
      default: return "badge badge-neutral";
    }
  }
</script>

<div class="flex flex-col overflow-hidden h-full">
  <!-- Project selector -->
  <div class="px-4 py-3 border-b" style="border-color: var(--border-color)">
    {#if memoryStore.projects.length === 0}
      <span class="text-xs" style="color: var(--text-muted)">{t("memory.noProjects")}</span>
    {:else}
      <select
        class="input-base w-full text-xs disabled:cursor-not-allowed disabled:opacity-60"
        style="padding: 0.375rem 0.5rem"
        value={memoryStore.activeProjectId ?? ""}
        disabled={isProjectScope}
        title={isProjectScope ? t("memory.dropdownDisabledInProjectScope") : ""}
        onchange={(e) => {
          const val = (e.target as HTMLSelectElement).value;
          if (val) memoryStore.selectProject(val);
        }}
      >
        <option value="">{t("memory.selectProject")}</option>
        {#each memoryStore.projects as project (project.id)}
          <option value={project.id}>{project.projectPath}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- File list -->
  <ul class="flex-1 overflow-y-auto py-2">
    {#if memoryStore.loading && memoryStore.activeProjectId && memoryStore.files.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">{t("common.loading")}</li>
    {:else if activeProjectHasNoMemory}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">
        {t("memory.noFilesYet", {
          name: projectsStore.activeProject?.path.replace(/^.*\//, "") ?? "",
        })}
      </li>
    {:else if !memoryStore.activeProjectId}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">{t("memory.selectProjectAbove")}</li>
    {:else if memoryStore.files.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">{t("memory.noFiles")}</li>
    {:else}
      {#each memoryStore.files as file (file.filename)}
        <li>
          {#if memoryStore.activeFile?.filename === file.filename}
            <button
              class="flex w-full flex-col gap-0.5 px-4 py-2 text-left transition-colors"
              style="background-color: var(--accent-bg); color: var(--text-primary)"
              onclick={() => memoryStore.selectFile(file.filename)}
            >
              <div class="flex items-center justify-between gap-2">
                <span class="flex items-center gap-1.5 truncate text-sm">
                  {#if memoryStore.activeFileDirty}
                    <span
                      class="inline-block h-1.5 w-1.5 flex-shrink-0 rounded-full"
                      style="background-color: var(--dirty-dot)"
                      aria-label={t("common.unsavedChanges")}
                    ></span>
                  {/if}
                  <span class="truncate">{file.name ?? file.filename}</span>
                </span>
                {#if file.memoryType}
                  <span class="flex-shrink-0 {typeBadgeClass(file.memoryType)}">
                    {file.memoryType}
                  </span>
                {/if}
              </div>
              {#if file.name && file.filename !== file.name}
                <span class="truncate font-mono text-xs" style="color: var(--text-muted)">{file.filename}</span>
              {/if}
            </button>
          {:else}
            <button
              class="flex w-full flex-col gap-0.5 px-4 py-2 text-left transition-colors hover:bg-[var(--bg-card-hover)]"
              style="color: var(--text-secondary)"
              onclick={() => memoryStore.selectFile(file.filename)}
            >
              <div class="flex items-center justify-between gap-2">
                <span class="flex items-center gap-1.5 truncate text-sm">
                  <span class="truncate">{file.name ?? file.filename}</span>
                </span>
                {#if file.memoryType}
                  <span class="flex-shrink-0 {typeBadgeClass(file.memoryType)}">
                    {file.memoryType}
                  </span>
                {/if}
              </div>
              {#if file.name && file.filename !== file.name}
                <span class="truncate font-mono text-xs" style="color: var(--text-muted)">{file.filename}</span>
              {/if}
            </button>
          {/if}
        </li>
      {/each}
    {/if}
  </ul>

  {#if memoryStore.error}
    <div class="px-4 py-2 text-xs border-t" style="color: var(--status-error-text); border-color: var(--border-color)">
      {memoryStore.error}
    </div>
  {/if}
</div>

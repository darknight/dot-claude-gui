<script lang="ts">
  import { memoryStore } from "$lib/stores/memory.svelte";
  import type { MemoryFile, MemoryProject } from "$lib/api/types";
  import { t } from "$lib/i18n";

  // Show only projects that have at least one memory file. Sort by
  // fileCount desc so heavier projects float to the top — useful when many
  // projects exist but only a handful are actively annotated.
  const visibleProjects = $derived(
    [...memoryStore.projects]
      .filter((p) => p.fileCount > 0)
      .sort((a, b) => b.fileCount - a.fileCount || a.projectPath.localeCompare(b.projectPath)),
  );

  function sortFiles(files: MemoryFile[]): MemoryFile[] {
    // MEMORY.md is the index file — pin it to the top of each group.
    return [...files].sort((a, b) => {
      if (a.filename === "MEMORY.md") return -1;
      if (b.filename === "MEMORY.md") return 1;
      return a.filename.localeCompare(b.filename);
    });
  }

  function typeBadgeClass(memoryType?: string): string {
    switch (memoryType) {
      case "core": return "badge badge-info";
      case "project": return "badge badge-purple";
      case "session": return "badge badge-warning";
      default: return "badge badge-neutral";
    }
  }

  function projectLabel(p: MemoryProject): string {
    return p.projectPath || p.id;
  }
</script>

<div class="flex flex-col overflow-hidden h-full">
  <ul class="flex-1 overflow-y-auto py-2">
    {#if visibleProjects.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">
        {memoryStore.projects.length === 0
          ? t("memory.noProjects")
          : t("memory.noProjectsWithMemory")}
      </li>
    {:else}
      {#each visibleProjects as project, groupIndex (project.id)}
        {@const isExpanded = memoryStore.expanded[project.id] ?? false}
        {@const files = memoryStore.filesByProject[project.id]}
        <li class={groupIndex === 0 ? "" : "mt-1"}>
          <button
            type="button"
            class="flex w-full items-center gap-1 px-3 py-1.5 text-left text-xs font-semibold tracking-wider hover:opacity-80"
            style="color: var(--text-muted)"
            onclick={() => void memoryStore.toggleProject(project.id)}
            title={project.projectPath}
          >
            <span class="inline-block w-3 flex-shrink-0 text-center">{isExpanded ? "▾" : "▸"}</span>
            <span class="truncate flex-1">{projectLabel(project)}</span>
            <span class="flex-shrink-0" style="color: var(--text-muted)">({project.fileCount})</span>
          </button>
        </li>
        {#if isExpanded}
          {#if files === undefined}
            <li class="px-7 py-1 text-xs" style="color: var(--text-muted)">{t("common.loading")}</li>
          {:else if files.length === 0}
            <li class="px-7 py-1 text-xs" style="color: var(--text-muted)">{t("memory.noFiles")}</li>
          {:else}
            {#each sortFiles(files) as file (file.filename)}
              {@const isActive = memoryStore.activeProjectId === project.id
                && memoryStore.activeFile?.filename === file.filename}
              <li>
                {#if isActive}
                  <button
                    class="flex w-full flex-col gap-0.5 py-1.5 pl-7 pr-4 text-left transition-colors"
                    style="background-color: var(--accent-bg); color: var(--text-primary)"
                    onclick={() => void memoryStore.selectFile(project.id, file.filename)}
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
                    class="flex w-full flex-col gap-0.5 py-1.5 pl-7 pr-4 text-left transition-colors hover:bg-[var(--bg-card-hover)]"
                    style="color: var(--text-secondary)"
                    onclick={() => void memoryStore.selectFile(project.id, file.filename)}
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
        {/if}
      {/each}
    {/if}
  </ul>

  {#if memoryStore.error}
    <div class="px-4 py-2 text-xs border-t" style="color: var(--status-error-text); border-color: var(--border-color)">
      {memoryStore.error}
    </div>
  {/if}
</div>

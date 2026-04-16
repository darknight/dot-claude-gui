<script lang="ts">
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { configStore } from "$lib/stores/config.svelte";

  const isProjectScope = $derived(
    configStore.activeScope === "project" && projectsStore.activeProjectId !== null,
  );

  // Files filtered by the current scope — only show the entry that matches
  // so the user can't accidentally edit an out-of-scope CLAUDE.md.
  const visibleFiles = $derived.by(() => {
    if (isProjectScope) {
      return claudeMdStore.files.filter(
        (f) =>
          f.scope === "project" &&
          f.projectId === projectsStore.activeProjectId,
      );
    }
    return claudeMdStore.files.filter((f) => f.scope === "global");
  });

  // Auto-select the single visible entry whenever scope changes.
  $effect(() => {
    const target = visibleFiles[0];
    if (target && claudeMdStore.activeFile?.id !== target.id) {
      claudeMdStore.selectFile(target.id);
    } else if (!target && claudeMdStore.activeFile) {
      claudeMdStore.activeFile = null;
    }
  });

  function scopeBadgeClass(scope: string): string {
    return scope === "global" ? "badge badge-info" : "badge badge-purple";
  }
</script>

<div class="flex flex-col overflow-hidden h-full">
  <ul class="flex-1 overflow-y-auto py-2">
    {#if claudeMdStore.loading && claudeMdStore.files.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">Loading...</li>
    {:else if visibleFiles.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">No CLAUDE.md files for this scope</li>
    {:else}
      <li class="px-4 pt-2 pb-1">
        <span class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
          {isProjectScope ? "Project" : "Global"}
        </span>
      </li>
      {#each visibleFiles as file (file.id)}
        <li>
          {#if claudeMdStore.activeFile?.id === file.id}
            <button
              class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors"
              style="background-color: var(--accent-bg); color: var(--text-primary)"
              onclick={() => claudeMdStore.selectFile(file.id)}
            >
              <span class="flex items-center gap-1.5 truncate">
                {#if claudeMdStore.activeFileDirty}
                  <span
                    class="inline-block h-1.5 w-1.5 flex-shrink-0 rounded-full"
                    style="background-color: var(--dirty-dot)"
                    aria-label="unsaved changes"
                  ></span>
                {/if}
                <span class="truncate">
                  {file.scope === "global"
                    ? (file.exists ? "CLAUDE.md" : "CLAUDE.md (create)")
                    : (file.projectName ?? file.projectId)}
                </span>
              </span>
              {#if !file.exists}
                <span class="ml-auto flex-shrink-0 text-xs" style="color: var(--text-muted)">click to create</span>
              {:else}
                <span class="ml-auto flex-shrink-0 {scopeBadgeClass(file.scope)}">
                  {file.scope === "global" ? "全局" : "项目"}
                </span>
              {/if}
            </button>
          {:else if file.exists}
            <button
              class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
              style="color: var(--text-secondary)"
              onclick={() => claudeMdStore.selectFile(file.id)}
            >
              <span class="flex items-center gap-1.5 truncate">
                <span class="truncate">
                  {file.scope === "global"
                    ? (file.exists ? "CLAUDE.md" : "CLAUDE.md (create)")
                    : (file.projectName ?? file.projectId)}
                </span>
              </span>
              <span class="ml-auto flex-shrink-0 {scopeBadgeClass(file.scope)}">
                {file.scope === "global" ? "全局" : "项目"}
              </span>
            </button>
          {:else}
            <button
              class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
              style="color: var(--text-muted); font-style: italic"
              onclick={() => claudeMdStore.selectFile(file.id)}
            >
              <span class="flex items-center gap-1.5 truncate">
                <span class="truncate">
                  {file.scope === "global" ? "CLAUDE.md (create)" : (file.projectName ?? file.projectId)}
                </span>
              </span>
              <span class="ml-auto flex-shrink-0 text-xs" style="color: var(--text-muted)">click to create</span>
            </button>
          {/if}
        </li>
      {/each}
    {/if}
  </ul>

  {#if claudeMdStore.error}
    <div class="border-t px-4 py-2 text-xs" style="color: var(--status-error-text); border-color: var(--border-color)">
      {claudeMdStore.error}
    </div>
  {/if}
</div>

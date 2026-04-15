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
    return scope === "global"
      ? "bg-blue-900 text-blue-300"
      : "bg-purple-900 text-purple-300";
  }
</script>

<div class="flex flex-col overflow-hidden h-full">
  <ul class="flex-1 overflow-y-auto py-2">
    {#if claudeMdStore.loading && claudeMdStore.files.length === 0}
      <li class="px-4 py-2 text-xs text-gray-500">Loading...</li>
    {:else if visibleFiles.length === 0}
      <li class="px-4 py-2 text-xs text-gray-600">No CLAUDE.md files for this scope</li>
    {:else}
      <li class="px-4 pt-2 pb-1">
        <span class="text-xs font-semibold uppercase tracking-wider text-gray-600">
          {isProjectScope ? "Project" : "Global"}
        </span>
      </li>
      {#each visibleFiles as file (file.id)}
        <li>
          <button
            class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors
              {claudeMdStore.activeFile?.id === file.id
              ? 'bg-gray-800 text-white'
              : file.exists
                ? 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'
                : 'text-gray-600 italic hover:bg-gray-800/50 hover:text-gray-400'}"
            onclick={() => claudeMdStore.selectFile(file.id)}
          >
            <span class="truncate">
              {file.scope === "global"
                ? (file.exists ? "CLAUDE.md" : "CLAUDE.md (create)")
                : (file.projectName ?? file.projectId)}
            </span>
            {#if !file.exists}
              <span class="ml-auto flex-shrink-0 text-xs text-gray-600">click to create</span>
            {:else}
              <span class="ml-auto flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {scopeBadgeClass(file.scope)}">
                {file.scope === "global" ? "全局" : "项目"}
              </span>
            {/if}
          </button>
        </li>
      {/each}
    {/if}
  </ul>

  {#if claudeMdStore.error}
    <div class="px-4 py-2 text-xs text-red-400 border-t border-gray-800">
      {claudeMdStore.error}
    </div>
  {/if}
</div>

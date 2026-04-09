<script lang="ts">
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";

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
    {:else if claudeMdStore.files.length === 0}
      <li class="px-4 py-2 text-xs text-gray-600">No CLAUDE.md files found</li>
    {:else}
      <li class="px-4 pt-2 pb-1">
        <span class="text-xs font-semibold uppercase tracking-wider text-gray-600">Global</span>
      </li>
      {#each claudeMdStore.files.filter((f) => f.scope === "global") as file (file.id)}
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
            <span class="truncate">{file.exists ? "CLAUDE.md" : "CLAUDE.md (create)"}</span>
            <span class="ml-auto flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {scopeBadgeClass(file.scope)}">
              全局
            </span>
          </button>
        </li>
      {/each}

      {@const projectFiles = claudeMdStore.files.filter((f) => f.scope === "project")}
      {#if projectFiles.length > 0}
        <li class="px-4 pt-4 pb-1">
          <span class="text-xs font-semibold uppercase tracking-wider text-gray-600">Projects</span>
        </li>
        {#each projectFiles as file (file.id)}
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
              <span class="truncate">{file.projectName ?? file.projectId}</span>
              {#if !file.exists}
                <span class="ml-auto flex-shrink-0 text-xs text-gray-600">click to create</span>
              {/if}
            </button>
          </li>
        {/each}
      {/if}
    {/if}
  </ul>

  {#if claudeMdStore.error}
    <div class="px-4 py-2 text-xs text-red-400 border-t border-gray-800">
      {claudeMdStore.error}
    </div>
  {/if}
</div>

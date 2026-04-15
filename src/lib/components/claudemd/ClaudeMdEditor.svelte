<script lang="ts">
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";

  let localContent = $state("");
  let originalContent = $state("");

  $effect(() => {
    const file = claudeMdStore.activeFile;
    if (file) {
      localContent = file.content;
      originalContent = file.content;
    } else {
      localContent = "";
      originalContent = "";
    }
  });

  let isDirty = $derived(localContent !== originalContent);

  $effect(() => {
    claudeMdStore.activeFileDirty = isDirty;
  });

  async function handleSave() {
    const file = claudeMdStore.activeFile;
    if (!file) return;
    await claudeMdStore.saveFile(file.id, localContent);
    originalContent = localContent;
  }

  async function handleDelete() {
    const file = claudeMdStore.activeFile;
    if (!file) return;
    if (!confirm("Are you sure you want to delete this CLAUDE.md?")) return;
    await claudeMdStore.deleteFile(file.id);
  }

  function scopeBadgeClass(scope: string): string {
    return scope === "global"
      ? "bg-blue-900 text-blue-300"
      : "bg-purple-900 text-purple-300";
  }

  function scopeLabel(scope: string): string {
    return scope === "global" ? "全局" : "项目";
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !claudeMdStore.activeFile}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Select a CLAUDE.md file to view and edit</p>
    </div>
  {:else}
    {@const file = claudeMdStore.activeFile}
    <div class="border-b border-gray-800 px-6 py-4">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="truncate text-sm font-semibold text-gray-100">
              {file.filename}
            </h2>
            <span class="flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {scopeBadgeClass(file.scope)}">
              {scopeLabel(file.scope)}
            </span>
            {#if isDirty}
              <span class="flex-shrink-0 rounded bg-orange-900 px-1.5 py-0.5 text-xs font-medium text-orange-300">
                unsaved
              </span>
            {/if}
          </div>
          <p class="mt-0.5 font-mono text-xs text-gray-500">{file.path}</p>
        </div>

        <div class="flex flex-shrink-0 items-center gap-2">
          <button
            class="rounded px-3 py-1.5 text-xs font-medium transition-colors
              {isDirty && !claudeMdStore.saving
              ? 'bg-blue-600 text-white hover:bg-blue-500'
              : 'cursor-not-allowed bg-gray-700 text-gray-500'}"
            disabled={!isDirty || claudeMdStore.saving}
            onclick={handleSave}
          >
            {claudeMdStore.saving ? "Saving..." : "Save"}
          </button>
          {#if file.scope !== "global" || originalContent !== ""}
            <button
              class="rounded px-3 py-1.5 text-xs font-medium text-red-400 transition-colors hover:bg-red-900/50 hover:text-red-300"
              onclick={handleDelete}
            >
              Delete
            </button>
          {/if}
        </div>
      </div>
    </div>

    <div class="flex flex-1 flex-col overflow-hidden p-4">
      {#if claudeMdStore.loading}
        <div class="flex flex-1 items-center justify-center">
          <p class="text-sm text-gray-500">Loading...</p>
        </div>
      {:else}
        <textarea
          class="flex-1 resize-none rounded border border-gray-700 bg-gray-950 p-3 font-mono text-xs text-gray-200 leading-relaxed focus:border-gray-600 focus:outline-none"
          bind:value={localContent}
          spellcheck={false}
        ></textarea>
      {/if}
    </div>

    {#if claudeMdStore.error}
      <div class="border-t border-gray-800 px-6 py-2 text-xs text-red-400">
        {claudeMdStore.error}
      </div>
    {/if}
  {/if}
</div>

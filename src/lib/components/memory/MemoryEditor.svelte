<script lang="ts">
  import { memoryStore } from "$lib/stores/memory.svelte";

  // Local editable content — tracks dirty state against the saved original
  let localContent = $state("");
  let originalContent = $state("");

  // Sync local content whenever the active file changes
  $effect(() => {
    const file = memoryStore.activeFile;
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
    memoryStore.activeFileDirty = isDirty;
  });

  async function handleSave() {
    const file = memoryStore.activeFile;
    const projectId = memoryStore.activeProjectId;
    if (!file || !projectId) return;
    await memoryStore.saveFile(projectId, file.filename, localContent);
    // After save, update originalContent so dirty state resets
    originalContent = localContent;
  }

  async function handleDelete() {
    const file = memoryStore.activeFile;
    const projectId = memoryStore.activeProjectId;
    if (!file || !projectId) return;
    if (!confirm(`Are you sure you want to delete "${file.filename}"?`)) return;
    await memoryStore.deleteFile(projectId, file.filename);
  }

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

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !memoryStore.activeFile}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Select a memory file to view and edit</p>
    </div>
  {:else}
    {@const file = memoryStore.activeFile}
    <!-- Header -->
    <div class="border-b border-gray-800 px-6 py-4">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="truncate text-sm font-semibold text-gray-100">
              {file.name ?? file.filename}
            </h2>
            {#if file.memoryType}
              <span class="flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {typeBadgeClass(file.memoryType)}">
                {file.memoryType}
              </span>
            {/if}
            {#if isDirty}
              <span class="flex-shrink-0 rounded bg-orange-900 px-1.5 py-0.5 text-xs font-medium text-orange-300">
                unsaved
              </span>
            {/if}
          </div>
          {#if file.name && file.filename !== file.name}
            <p class="mt-0.5 font-mono text-xs text-gray-500">{file.filename}</p>
          {/if}
          {#if file.description}
            <p class="mt-1 text-xs text-gray-400">{file.description}</p>
          {/if}
        </div>

        <!-- Action buttons -->
        <div class="flex flex-shrink-0 items-center gap-2">
          <button
            class="rounded px-3 py-1.5 text-xs font-medium transition-colors
              {isDirty && !memoryStore.saving
              ? 'bg-blue-600 text-white hover:bg-blue-500'
              : 'cursor-not-allowed bg-gray-700 text-gray-500'}"
            disabled={!isDirty || memoryStore.saving}
            onclick={handleSave}
          >
            {memoryStore.saving ? "Saving..." : "Save"}
          </button>
          <button
            class="rounded px-3 py-1.5 text-xs font-medium text-red-400 transition-colors hover:bg-red-900/50 hover:text-red-300"
            onclick={handleDelete}
          >
            Delete
          </button>
        </div>
      </div>
    </div>

    <!-- Editor -->
    <div class="flex flex-1 flex-col overflow-hidden p-4">
      {#if memoryStore.loading}
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

    {#if memoryStore.error}
      <div class="border-t border-gray-800 px-6 py-2 text-xs text-red-400">
        {memoryStore.error}
      </div>
    {/if}
  {/if}
</div>

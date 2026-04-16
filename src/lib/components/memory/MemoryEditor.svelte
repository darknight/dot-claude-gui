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
      case "core": return "badge badge-info";
      case "project": return "badge badge-purple";
      case "session": return "badge badge-warning";
      default: return "badge badge-neutral";
    }
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !memoryStore.activeFile}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">Select a memory file to view and edit</p>
    </div>
  {:else}
    {@const file = memoryStore.activeFile}
    <!-- Header -->
    <div class="border-b px-6 py-4" style="border-color: var(--border-color)">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="truncate text-sm font-semibold" style="color: var(--text-primary)">
              {file.name ?? file.filename}
            </h2>
            {#if file.memoryType}
              <span class="flex-shrink-0 {typeBadgeClass(file.memoryType)}">
                {file.memoryType}
              </span>
            {/if}
            {#if isDirty}
              <span class="badge badge-warning flex-shrink-0">
                unsaved
              </span>
            {/if}
          </div>
          {#if file.name && file.filename !== file.name}
            <p class="mt-0.5 font-mono text-xs" style="color: var(--text-muted)">{file.filename}</p>
          {/if}
          {#if file.description}
            <p class="mt-1 text-xs" style="color: var(--text-secondary)">{file.description}</p>
          {/if}
        </div>

        <!-- Action buttons -->
        <div class="flex flex-shrink-0 items-center gap-2">
          <button
            class="btn-primary rounded px-3 py-1.5 text-xs font-medium"
            disabled={!isDirty || memoryStore.saving}
            onclick={handleSave}
          >
            {memoryStore.saving ? "Saving..." : "Save"}
          </button>
          <button
            class="btn-danger-ghost rounded px-3 py-1.5 text-xs font-medium"
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
          <p class="text-sm" style="color: var(--text-muted)">Loading...</p>
        </div>
      {:else}
        <textarea
          class="flex-1 resize-none font-mono text-xs leading-relaxed focus:outline-none"
          style="background-color: var(--bg-code); color: var(--text-primary); border: 1px solid var(--border-color); border-radius: 0.25rem; padding: 0.75rem"
          bind:value={localContent}
          spellcheck={false}
        ></textarea>
      {/if}
    </div>

    {#if memoryStore.error}
      <div class="alert-error border-t px-6 py-2 text-xs" style="border-color: var(--border-color)">
        {memoryStore.error}
      </div>
    {/if}
  {/if}
</div>

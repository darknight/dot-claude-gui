<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);

  let entries = $state<{ key: string; value: string }[]>([]);

  $effect(() => {
    entries = Object.entries(settings.env ?? {}).map(([key, value]) => ({ key, value }));
  });

  const originalEnv = $derived(settings.env ?? {});
  function isRowDirty(key: string, value: string): boolean {
    if (!(key in originalEnv)) return true;
    return originalEnv[key] !== value;
  }

  let newKey = $state("");
  let newValue = $state("");
  let addError = $state("");

  function addEntry() {
    const trimmedKey = newKey.trim();
    if (!trimmedKey) {
      addError = "Key cannot be empty.";
      return;
    }
    if (entries.some((e) => e.key === trimmedKey)) {
      addError = `Key "${trimmedKey}" already exists.`;
      return;
    }
    entries = [...entries, { key: trimmedKey, value: newValue }];
    newKey = "";
    newValue = "";
    addError = "";
    configStore.markDirty();
  }

  function removeEntry(index: number) {
    entries = entries.filter((_, i) => i !== index);
    configStore.markDirty();
  }

  function updateValue(index: number, value: string) {
    entries = entries.map((e, i) => (i === index ? { ...e, value } : e));
    configStore.markDirty();
  }

  const envObject = $derived(Object.fromEntries(entries.map((e) => [e.key, e.value])));

  function save() {
    configStore.save({ env: envObject });
  }

  function handleAddKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      addEntry();
    }
  }

  const previewData = $derived({ env: envObject });
</script>

<div class="space-y-5 max-w-2xl">
  <div class="space-y-1">
    <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">Environment Variables</h3>
    <p class="text-xs text-gray-500 dark:text-gray-400">
      Key-value pairs passed as environment variables to Claude Code.
    </p>
  </div>

  <!-- Existing entries -->
  {#if entries.length > 0}
    <div class="space-y-2">
      {#each entries as entry, index}
        {@const rowDirty = isRowDirty(entry.key, entry.value)}
        <div class="group relative flex items-center gap-2">
          {#if rowDirty}
            <span
              class="absolute -left-3 top-1/2 -translate-y-1/2 inline-block h-1.5 w-1.5 rounded-full bg-orange-500"
              aria-label="Modified"
            ></span>
          {/if}
          <code class="shrink-0 rounded bg-gray-100 dark:bg-gray-800 px-2 py-1.5 text-xs font-mono text-gray-800 dark:text-gray-200 border border-gray-200 dark:border-gray-700">
            {entry.key}
          </code>
          <span class="text-gray-400 text-sm shrink-0">=</span>
          <input
            type="text"
            value={entry.value}
            oninput={(e) => updateValue(index, (e.target as HTMLInputElement).value)}
            class="flex-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="button"
            onclick={() => removeEntry(index)}
            class="shrink-0 text-xs text-red-400 opacity-0 group-hover:opacity-100 hover:text-red-600 transition-opacity"
            aria-label="Remove {entry.key}"
          >
            Remove
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-gray-400 dark:text-gray-500 italic">No environment variables defined.</p>
  {/if}

  <!-- Add new entry -->
  <div class="space-y-2 border-t border-gray-200 dark:border-gray-700 pt-4">
    <span class="block text-xs font-medium text-gray-600 dark:text-gray-400">Add New Variable</span>
    <div class="flex items-center gap-2">
      <input
        type="text"
        bind:value={newKey}
        onkeydown={handleAddKeydown}
        oninput={() => { addError = ""; }}
        placeholder="KEY"
        class="w-40 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm font-mono text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <span class="text-gray-400 text-sm shrink-0">=</span>
      <input
        type="text"
        bind:value={newValue}
        onkeydown={handleAddKeydown}
        placeholder="value"
        class="flex-1 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <button
        type="button"
        onclick={addEntry}
        class="shrink-0 text-sm px-3 py-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40"
      >
        Add
      </button>
    </div>
    {#if addError}
      <p class="text-xs text-red-500">{addError}</p>
    {/if}
  </div>

  <!-- Save / Revert -->
  <div class="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
    <button
      type="button"
      onclick={save}
      disabled={!configStore.isDirty || configStore.saving}
      class="text-sm px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40"
    >
      {configStore.saving ? "Saving..." : "Save"}
    </button>
    <button
      type="button"
      onclick={() => configStore.revert()}
      disabled={!configStore.isDirty}
      class="text-sm px-4 py-2 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40"
    >
      Revert
    </button>
  </div>

  <!-- JSON Preview -->
  <JsonPreview data={previewData} title="Environment Variables (JSON)" />
</div>

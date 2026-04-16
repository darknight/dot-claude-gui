<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";

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
    <h3 class="text-sm font-medium" style="color: var(--text-secondary)">Environment Variables</h3>
    <p class="text-xs" style="color: var(--text-muted)">
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
              class="absolute -left-3 top-1/2 -translate-y-1/2 inline-block h-1.5 w-1.5 rounded-full"
              style="background-color: var(--dirty-dot)"
              aria-label="Modified"
            ></span>
          {/if}
          <code class="shrink-0 rounded px-2 py-1.5 text-xs font-mono" style="background-color: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border-color)">
            {entry.key}
          </code>
          <span class="text-sm shrink-0" style="color: var(--text-muted)">=</span>
          <input
            type="text"
            value={entry.value}
            oninput={(e) => updateValue(index, (e.target as HTMLInputElement).value)}
            class="input-base flex-1"
          />
          <button
            type="button"
            onclick={() => removeEntry(index)}
            class="btn-danger-ghost shrink-0 text-xs opacity-0 group-hover:opacity-100 transition-opacity"
            aria-label="Remove {entry.key}"
          >
            {t("common.remove")}
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-sm italic" style="color: var(--text-muted)">{t("settings.noEnvVars")}</p>
  {/if}

  <!-- Add new entry -->
  <div class="space-y-2 border-t pt-4" style="border-color: var(--border-color)">
    <span class="block text-xs font-medium" style="color: var(--text-muted)">Add New Variable</span>
    <div class="flex items-center gap-2">
      <input
        type="text"
        bind:value={newKey}
        onkeydown={handleAddKeydown}
        oninput={() => { addError = ""; }}
        placeholder="KEY"
        class="input-base w-40 font-mono"
      />
      <span class="text-sm shrink-0" style="color: var(--text-muted)">=</span>
      <input
        type="text"
        bind:value={newValue}
        onkeydown={handleAddKeydown}
        placeholder="value"
        class="input-base flex-1"
      />
      <button
        type="button"
        onclick={addEntry}
        class="btn-primary shrink-0 text-sm px-3 py-1.5"
      >
        {t("common.add")}
      </button>
    </div>
    {#if addError}
      <p class="text-xs" style="color: var(--status-error-text)">{addError}</p>
    {/if}
  </div>

  <!-- Save / Revert -->
  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button
      type="button"
      onclick={save}
      disabled={!configStore.isDirty || configStore.saving}
      class="btn-primary text-sm px-4 py-2"
    >
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button
      type="button"
      onclick={() => configStore.revert()}
      disabled={!configStore.isDirty}
      class="btn-secondary text-sm px-4 py-2"
    >
      {t("common.revert")}
    </button>
  </div>

  <!-- JSON Preview -->
  <JsonPreview data={previewData} title="Environment Variables (JSON)" />
</div>

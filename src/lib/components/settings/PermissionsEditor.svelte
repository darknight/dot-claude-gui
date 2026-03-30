<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import StringListEditor from "$lib/components/shared/StringListEditor.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);
  const perms = $derived(settings.permissions);

  let allow = $state<string[]>([]);
  let deny = $state<string[]>([]);
  let ask = $state<string[]>([]);
  let defaultMode = $state("default");

  let initialized = $state(false);

  // Sync from store when settings change
  $effect(() => {
    allow = [...(perms?.allow ?? [])];
    deny = [...(perms?.deny ?? [])];
    ask = [...(perms?.ask ?? [])];
    defaultMode = perms?.defaultMode ?? "default";
    initialized = true;
  });

  // Mark dirty when list contents change (after initial sync)
  $effect(() => {
    void allow.length;
    void deny.length;
    void ask.length;
    if (initialized) configStore.markDirty();
  });

  const previewData = $derived({
    permissions: { allow, deny, ask, defaultMode },
  });

  function save() {
    configStore.save({ permissions: { allow, deny, ask, defaultMode } });
  }
</script>

<div class="space-y-5 max-w-xl">
  <!-- Allow -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={allow}
      label="Allow"
      placeholder="e.g. Bash(git:*)"
    />
  </div>

  <!-- Deny -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={deny}
      label="Deny"
      placeholder="e.g. Bash(rm:*)"
    />
  </div>

  <!-- Ask -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={ask}
      label="Ask"
      placeholder="e.g. WebSearch"
    />
  </div>

  <!-- Default Mode -->
  <div class="space-y-1">
    <label
      for="defaultMode"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Default Mode
    </label>
    <select
      id="defaultMode"
      bind:value={defaultMode}
      onchange={() => configStore.markDirty()}
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      <option value="acceptEdits">acceptEdits</option>
      <option value="bypassPermissions">bypassPermissions</option>
      <option value="default">default</option>
      <option value="dontAsk">dontAsk</option>
      <option value="plan">plan</option>
      <option value="auto">auto</option>
    </select>
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
  <JsonPreview data={previewData} title="Permissions (JSON)" />
</div>

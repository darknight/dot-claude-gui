<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);

  let language = $state(settings.language ?? "");
  let alwaysThinkingEnabled = $state(settings.alwaysThinkingEnabled ?? false);
  let autoUpdatesChannel = $state(settings.autoUpdatesChannel ?? "stable");
  let minimumVersion = $state(settings.minimumVersion ?? "");
  let includeCoAuthoredBy = $state(settings.includeCoAuthoredBy ?? false);
  let skipDangerousModePermissionPrompt = $state(
    settings.skipDangerousModePermissionPrompt ?? false,
  );

  $effect(() => {
    language = settings.language ?? "";
    alwaysThinkingEnabled = settings.alwaysThinkingEnabled ?? false;
    autoUpdatesChannel = settings.autoUpdatesChannel ?? "stable";
    minimumVersion = settings.minimumVersion ?? "";
    includeCoAuthoredBy = settings.includeCoAuthoredBy ?? false;
    skipDangerousModePermissionPrompt =
      settings.skipDangerousModePermissionPrompt ?? false;
  });

  const previewData = $derived({
    language: language || undefined,
    alwaysThinkingEnabled,
    autoUpdatesChannel,
    minimumVersion: minimumVersion || undefined,
    includeCoAuthoredBy,
    skipDangerousModePermissionPrompt,
  });

  function save() {
    configStore.save({
      language: language || undefined,
      alwaysThinkingEnabled,
      autoUpdatesChannel,
      minimumVersion: minimumVersion || undefined,
      includeCoAuthoredBy,
      skipDangerousModePermissionPrompt,
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <!-- Language -->
  <div class="space-y-1">
    <label
      for="language"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Language
    </label>
    <input
      id="language"
      type="text"
      bind:value={language}
      oninput={() => configStore.markDirty()}
      placeholder="e.g. Simplified Chinese"
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
  </div>

  <!-- Always Thinking Enabled -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={alwaysThinkingEnabled}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-500 focus:ring-blue-500"
    />
    <span class="text-sm text-gray-700 dark:text-gray-300">
      Always thinking enabled
    </span>
  </label>

  <!-- Auto Updates Channel -->
  <div class="space-y-1">
    <label
      for="autoUpdatesChannel"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Auto Updates Channel
    </label>
    <select
      id="autoUpdatesChannel"
      bind:value={autoUpdatesChannel}
      onchange={() => configStore.markDirty()}
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      <option value="stable">stable</option>
      <option value="latest">latest</option>
    </select>
  </div>

  <!-- Minimum Version -->
  <div class="space-y-1">
    <label
      for="minimumVersion"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Minimum Version
    </label>
    <input
      id="minimumVersion"
      type="text"
      bind:value={minimumVersion}
      oninput={() => configStore.markDirty()}
      placeholder="e.g. 2.1.63"
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
  </div>

  <!-- Include Co-authored-by -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={includeCoAuthoredBy}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-500 focus:ring-blue-500"
    />
    <span class="text-sm text-gray-700 dark:text-gray-300">
      Include Co-authored-by in commits
    </span>
  </label>

  <!-- Skip Dangerous Mode Permission Prompt -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={skipDangerousModePermissionPrompt}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-500 focus:ring-blue-500"
    />
    <span class="text-sm text-gray-700 dark:text-gray-300">
      Skip dangerous mode permission prompt
    </span>
  </label>

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
  <JsonPreview data={previewData} title="General Settings (JSON)" />
</div>

<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);
  const statusLine = $derived(settings.statusLine ?? {});

  let statusType = $state(statusLine.type ?? "command");
  let command = $state(statusLine.command ?? "");
  let padding = $state(statusLine.padding ?? 0);

  $effect(() => {
    statusType = statusLine.type ?? "command";
    command = statusLine.command ?? "";
    padding = statusLine.padding ?? 0;
  });

  const previewData = $derived({
    statusLine: {
      type: statusType,
      command: command || undefined,
      padding,
    },
  });

  function save() {
    configStore.save({
      statusLine: {
        type: statusType,
        command: command || undefined,
        padding,
      },
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <!-- Type -->
  <div class="space-y-1">
    <label
      for="statusType"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Type
    </label>
    <select
      id="statusType"
      bind:value={statusType}
      onchange={() => configStore.markDirty()}
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      <option value="command">command</option>
    </select>
  </div>

  <!-- Command -->
  <div class="space-y-1">
    <label
      for="command"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Command
    </label>
    <input
      id="command"
      type="text"
      bind:value={command}
      oninput={() => configStore.markDirty()}
      placeholder="e.g. bunx -y ccstatusline@latest"
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm font-mono text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
  </div>

  <!-- Padding -->
  <div class="space-y-1">
    <label
      for="padding"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Padding
    </label>
    <input
      id="padding"
      type="number"
      min="0"
      bind:value={padding}
      oninput={() => configStore.markDirty()}
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
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
  <JsonPreview data={previewData} title="Status Line (JSON)" />
</div>

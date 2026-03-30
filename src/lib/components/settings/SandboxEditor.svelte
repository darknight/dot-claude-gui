<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import StringListEditor from "$lib/components/shared/StringListEditor.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);
  const sandbox = $derived(settings.sandbox);

  let allowRead = $state<string[]>([]);
  let denyRead = $state<string[]>([]);
  let allowWrite = $state<string[]>([]);
  let excludedCommands = $state<string[]>([]);
  let failIfUnavailable = $state(false);
  let enableWeakerNetworkIsolation = $state(false);

  let initialized = $state(false);

  // Sync from store when settings change
  $effect(() => {
    allowRead = [...(sandbox?.allowRead ?? [])];
    denyRead = [...(sandbox?.denyRead ?? [])];
    allowWrite = [...(sandbox?.allowWrite ?? [])];
    excludedCommands = [...(sandbox?.excludedCommands ?? [])];
    failIfUnavailable = sandbox?.failIfUnavailable ?? false;
    enableWeakerNetworkIsolation = sandbox?.enableWeakerNetworkIsolation ?? false;
    initialized = true;
  });

  // Mark dirty when list contents change (after initial sync)
  $effect(() => {
    void allowRead.length;
    void denyRead.length;
    void allowWrite.length;
    void excludedCommands.length;
    if (initialized) configStore.markDirty();
  });

  const previewData = $derived({
    sandbox: {
      allowRead,
      denyRead,
      allowWrite,
      excludedCommands,
      failIfUnavailable,
      enableWeakerNetworkIsolation,
    },
  });

  function save() {
    configStore.save({
      sandbox: {
        allowRead,
        denyRead,
        allowWrite,
        excludedCommands,
        failIfUnavailable,
        enableWeakerNetworkIsolation,
      },
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <!-- Allow Read -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={allowRead}
      label="Allow Read"
      placeholder="e.g. /home/user/docs"
    />
  </div>

  <!-- Deny Read -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={denyRead}
      label="Deny Read"
      placeholder="e.g. /etc/secrets"
    />
  </div>

  <!-- Allow Write -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={allowWrite}
      label="Allow Write"
      placeholder="e.g. /tmp/output"
    />
  </div>

  <!-- Excluded Commands -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={excludedCommands}
      label="Excluded Commands"
      placeholder="e.g. curl"
    />
  </div>

  <!-- Fail If Unavailable -->
  <div class="flex items-center gap-3">
    <input
      id="failIfUnavailable"
      type="checkbox"
      bind:checked={failIfUnavailable}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-500 focus:ring-blue-500"
    />
    <label
      for="failIfUnavailable"
      class="text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Fail If Unavailable
    </label>
  </div>

  <!-- Enable Weaker Network Isolation -->
  <div class="flex items-center gap-3">
    <input
      id="enableWeakerNetworkIsolation"
      type="checkbox"
      bind:checked={enableWeakerNetworkIsolation}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-500 focus:ring-blue-500"
    />
    <label
      for="enableWeakerNetworkIsolation"
      class="text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Enable Weaker Network Isolation
    </label>
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
  <JsonPreview data={previewData} title="Sandbox (JSON)" />
</div>

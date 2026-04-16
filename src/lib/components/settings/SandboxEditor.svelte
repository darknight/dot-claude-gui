<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import StringListEditor from "$lib/components/shared/StringListEditor.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import { arraysEqual } from "$lib/utils/diff";
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

  const allowReadDirty = $derived(!arraysEqual(allowRead, sandbox?.allowRead));
  const denyReadDirty = $derived(!arraysEqual(denyRead, sandbox?.denyRead));
  const allowWriteDirty = $derived(
    !arraysEqual(allowWrite, sandbox?.allowWrite),
  );
  const excludedCommandsDirty = $derived(
    !arraysEqual(excludedCommands, sandbox?.excludedCommands),
  );
  const failIfUnavailableDirty = $derived(
    failIfUnavailable !== (sandbox?.failIfUnavailable ?? false),
  );
  const weakerIsolationDirty = $derived(
    enableWeakerNetworkIsolation !==
      (sandbox?.enableWeakerNetworkIsolation ?? false),
  );

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
      dirty={allowReadDirty}
    />
  </div>

  <!-- Deny Read -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={denyRead}
      label="Deny Read"
      placeholder="e.g. /etc/secrets"
      dirty={denyReadDirty}
    />
  </div>

  <!-- Allow Write -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={allowWrite}
      label="Allow Write"
      placeholder="e.g. /tmp/output"
      dirty={allowWriteDirty}
    />
  </div>

  <!-- Excluded Commands -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={excludedCommands}
      label="Excluded Commands"
      placeholder="e.g. curl"
      dirty={excludedCommandsDirty}
    />
  </div>

  <!-- Fail If Unavailable -->
  <div class="flex items-center gap-3">
    <input
      id="failIfUnavailable"
      type="checkbox"
      bind:checked={failIfUnavailable}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <label
      for="failIfUnavailable"
      class="text-sm font-medium" style="color: var(--text-secondary)"
    >
      Fail If Unavailable
      <DirtyDot dirty={failIfUnavailableDirty} />
    </label>
  </div>

  <!-- Enable Weaker Network Isolation -->
  <div class="flex items-center gap-3">
    <input
      id="enableWeakerNetworkIsolation"
      type="checkbox"
      bind:checked={enableWeakerNetworkIsolation}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <label
      for="enableWeakerNetworkIsolation"
      class="text-sm font-medium" style="color: var(--text-secondary)"
    >
      Enable Weaker Network Isolation
      <DirtyDot dirty={weakerIsolationDirty} />
    </label>
  </div>

  <!-- Save / Revert -->
  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button
      type="button"
      onclick={save}
      disabled={!configStore.isDirty || configStore.saving}
      class="btn-primary text-sm px-4 py-2"
    >
      {configStore.saving ? "Saving..." : "Save"}
    </button>
    <button
      type="button"
      onclick={() => configStore.revert()}
      disabled={!configStore.isDirty}
      class="btn-secondary text-sm px-4 py-2"
    >
      Revert
    </button>
  </div>

  <!-- JSON Preview -->
  <JsonPreview data={previewData} title="Sandbox (JSON)" />
</div>

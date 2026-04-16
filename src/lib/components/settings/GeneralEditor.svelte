<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
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

  const languageDirty = $derived(language !== (settings.language ?? ""));
  const thinkingDirty = $derived(
    alwaysThinkingEnabled !== (settings.alwaysThinkingEnabled ?? false),
  );
  const channelDirty = $derived(
    autoUpdatesChannel !== (settings.autoUpdatesChannel ?? "stable"),
  );
  const versionDirty = $derived(
    minimumVersion !== (settings.minimumVersion ?? ""),
  );
  const coauthorDirty = $derived(
    includeCoAuthoredBy !== (settings.includeCoAuthoredBy ?? false),
  );
  const skipDangerousDirty = $derived(
    skipDangerousModePermissionPrompt !==
      (settings.skipDangerousModePermissionPrompt ?? false),
  );

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
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Language
      <DirtyDot dirty={languageDirty} />
    </label>
    <input
      id="language"
      type="text"
      bind:value={language}
      oninput={() => configStore.markDirty()}
      placeholder="e.g. Simplified Chinese"
      class="input-base"
    />
  </div>

  <!-- Always Thinking Enabled -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={alwaysThinkingEnabled}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <span class="text-sm" style="color: var(--text-secondary)">
      Always thinking enabled
      <DirtyDot dirty={thinkingDirty} />
    </span>
  </label>

  <!-- Auto Updates Channel -->
  <div class="space-y-1">
    <label
      for="autoUpdatesChannel"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Auto Updates Channel
      <DirtyDot dirty={channelDirty} />
    </label>
    <select
      id="autoUpdatesChannel"
      bind:value={autoUpdatesChannel}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="stable">stable</option>
      <option value="latest">latest</option>
    </select>
  </div>

  <!-- Minimum Version -->
  <div class="space-y-1">
    <label
      for="minimumVersion"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Minimum Version
      <DirtyDot dirty={versionDirty} />
    </label>
    <input
      id="minimumVersion"
      type="text"
      bind:value={minimumVersion}
      oninput={() => configStore.markDirty()}
      placeholder="e.g. 2.1.63"
      class="input-base"
    />
  </div>

  <!-- Include Co-authored-by -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={includeCoAuthoredBy}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <span class="text-sm" style="color: var(--text-secondary)">
      Include Co-authored-by in commits
      <DirtyDot dirty={coauthorDirty} />
    </span>
  </label>

  <!-- Skip Dangerous Mode Permission Prompt -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={skipDangerousModePermissionPrompt}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <span class="text-sm" style="color: var(--text-secondary)">
      Skip dangerous mode permission prompt
      <DirtyDot dirty={skipDangerousDirty} />
    </span>
  </label>

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
  <JsonPreview data={previewData} title="General Settings (JSON)" />
</div>

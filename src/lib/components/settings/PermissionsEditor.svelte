<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import StringListEditor from "$lib/components/shared/StringListEditor.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import { arraysEqual } from "$lib/utils/diff";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";

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

  const allowDirty = $derived(!arraysEqual(allow, perms?.allow));
  const denyDirty = $derived(!arraysEqual(deny, perms?.deny));
  const askDirty = $derived(!arraysEqual(ask, perms?.ask));
  const modeDirty = $derived(defaultMode !== (perms?.defaultMode ?? "default"));

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
      dirty={allowDirty}
    />
  </div>

  <!-- Deny -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={deny}
      label="Deny"
      placeholder="e.g. Bash(rm:*)"
      dirty={denyDirty}
    />
  </div>

  <!-- Ask -->
  <div class="space-y-1">
    <StringListEditor
      bind:items={ask}
      label="Ask"
      placeholder="e.g. WebSearch"
      dirty={askDirty}
    />
  </div>

  <!-- Default Mode -->
  <div class="space-y-1">
    <label
      for="defaultMode"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Default Mode
      <DirtyDot dirty={modeDirty} />
    </label>
    <select
      id="defaultMode"
      bind:value={defaultMode}
      onchange={() => configStore.markDirty()}
      class="input-base"
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
  <JsonPreview data={previewData} title="Permissions (JSON)" />
</div>

<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";

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

  const typeDirty = $derived(statusType !== (statusLine.type ?? "command"));
  const commandDirty = $derived(command !== (statusLine.command ?? ""));
  const paddingDirty = $derived(padding !== (statusLine.padding ?? 0));

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
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Type
      <DirtyDot dirty={typeDirty} />
    </label>
    <select
      id="statusType"
      bind:value={statusType}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="command">command</option>
    </select>
  </div>

  <!-- Command -->
  <div class="space-y-1">
    <label
      for="command"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Command
      <DirtyDot dirty={commandDirty} />
    </label>
    <input
      id="command"
      type="text"
      bind:value={command}
      oninput={() => configStore.markDirty()}
      placeholder="e.g. bunx -y ccstatusline@latest"
      class="input-base font-mono"
    />
  </div>

  <!-- Padding -->
  <div class="space-y-1">
    <label
      for="padding"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      Padding
      <DirtyDot dirty={paddingDirty} />
    </label>
    <input
      id="padding"
      type="number"
      min="0"
      bind:value={padding}
      oninput={() => configStore.markDirty()}
      class="input-base"
    />
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
  <JsonPreview data={previewData} title="Status Line (JSON)" />
</div>

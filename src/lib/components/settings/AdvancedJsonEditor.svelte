<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import SchemaKeyPicker from "./sub/SchemaKeyPicker.svelte";
  import JsonValueEditor from "./sub/JsonValueEditor.svelte";
  import { schemaSnapshot, type SchemaField } from "$lib/api/schema-snapshot";
  import { t } from "$lib/i18n";

  const settings = $derived(configStore.activeSettings);
  const settingsMap = $derived(settings as Record<string, unknown>);

  let selectedKey = $state<string | null>(null);
  const selectedField = $derived<SchemaField | null>(
    selectedKey
      ? (schemaSnapshot.settingsFields.find((f) => f.name === selectedKey) ?? {
          name: selectedKey,
          type: "unknown",
          describe: "Not in schema snapshot",
        })
      : null,
  );
  let pendingValue = $state<unknown>(undefined);

  $effect(() => {
    if (selectedKey) {
      pendingValue = settingsMap[selectedKey];
    } else {
      pendingValue = undefined;
    }
  });

  function saveField() {
    if (!selectedKey) return;
    configStore.save({ [selectedKey]: pendingValue });
  }

  function resetField() {
    if (!selectedKey) return;
    pendingValue = undefined;
    configStore.save({ [selectedKey]: undefined });
  }
</script>

<div class="flex gap-4 h-full">
  <aside class="w-80 shrink-0 border-r pr-4"
         style="border-color: var(--border-color)">
    <h3 class="text-sm font-semibold mb-3" style="color: var(--text-primary)">
      {t("settings.advanced.keyPicker")}
    </h3>
    <SchemaKeyPicker {settingsMap} bind:selected={selectedKey} />
  </aside>

  <section class="flex-1 min-w-0">
    {#if !selectedField}
      <p class="text-sm" style="color: var(--text-muted)">
        {t("settings.advanced.noSelection")}
      </p>
    {:else}
      <div class="space-y-3 max-w-2xl">
        <div>
          <h3 class="text-lg font-semibold font-mono" style="color: var(--text-primary)">
            {selectedField.name}
          </h3>
          <p class="text-xs font-mono" style="color: var(--text-muted)">
            {t("settings.advanced.typeLabel")}: {selectedField.type}
            {selectedField.enumValues
              ? " · enum(" + selectedField.enumValues.join(", ") + ")"
              : ""}
          </p>
        </div>

        {#if selectedField.describe}
          <div>
            <label class="block text-xs font-semibold uppercase mb-1"
                   style="color: var(--text-muted)">
              {t("settings.advanced.describeLabel")}
            </label>
            <p class="text-sm" style="color: var(--text-secondary)">
              {selectedField.describe}
            </p>
          </div>
        {/if}

        <div>
          <label class="block text-xs font-semibold uppercase mb-1"
                 style="color: var(--text-muted)">
            {t("settings.advanced.valueLabel")}
          </label>
          <JsonValueEditor
            field={selectedField}
            bind:value={pendingValue}
            onChange={() => configStore.markDirty()} />
        </div>

        <div class="flex gap-2 pt-3 border-t" style="border-color: var(--border-color)">
          <button type="button" onclick={saveField}
                  disabled={!configStore.isDirty || configStore.saving}
                  class="btn-primary text-sm px-4 py-2">
            {configStore.saving ? t("common.saving") : t("common.save")}
          </button>
          <button type="button" onclick={resetField}
                  class="btn-secondary text-sm px-4 py-2">
            {t("settings.advanced.resetButton")}
          </button>
        </div>

        <JsonPreview
          data={{ [selectedField.name]: pendingValue }}
          title="Current field" />
      </div>
    {/if}
  </section>
</div>

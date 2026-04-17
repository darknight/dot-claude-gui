<script lang="ts">
  import { t } from "$lib/i18n";
  import type { SchemaField } from "$lib/api/schema-snapshot";

  let {
    field,
    value = $bindable(),
    onChange,
  }: {
    field: SchemaField;
    value: unknown;
    onChange: () => void;
  } = $props();

  let textBuffer = $state("");
  let parseError = $state("");

  $effect(() => {
    if (field.type === "object" || field.type === "array" || field.type === "record") {
      textBuffer = value === undefined ? "" : JSON.stringify(value, null, 2);
      parseError = "";
    }
  });

  function syncFromText() {
    const trimmed = textBuffer.trim();
    if (!trimmed) {
      value = undefined;
      parseError = "";
      onChange();
      return;
    }
    try {
      value = JSON.parse(trimmed);
      parseError = "";
      onChange();
    } catch (e) {
      parseError = t("settings.advanced.invalidJson") + ": " + String(e);
    }
  }

  function strValue(): string {
    return value === undefined || value === null ? "" : String(value);
  }

  function numValue(): string {
    return typeof value === "number" ? String(value) : "";
  }

  function boolValue(): boolean {
    return Boolean(value);
  }
</script>

<div class="space-y-2">
  {#if field.type === "boolean"}
    <label class="flex items-center gap-3 cursor-pointer">
      <input type="checkbox" checked={boolValue()}
             onchange={(e) => { value = (e.currentTarget as HTMLInputElement).checked; onChange(); }}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" style="color: var(--text-secondary)">
        {boolValue() ? "on" : "off"}
      </span>
    </label>

  {:else if field.type === "enum"}
    <select value={strValue()}
            onchange={(e) => { value = (e.currentTarget as HTMLSelectElement).value || undefined; onChange(); }}
            class="input-base">
      <option value="">(unset)</option>
      {#each field.enumValues ?? [] as opt}
        <option value={opt}>{opt}</option>
      {/each}
    </select>

  {:else if field.type === "number"}
    <input type="number" value={numValue()}
           oninput={(e) => { const n = Number((e.currentTarget as HTMLInputElement).value); value = Number.isFinite(n) ? n : undefined; onChange(); }}
           class="input-base" />

  {:else if field.type === "string" || field.type === "literal"}
    <input type="text" value={strValue()}
           oninput={(e) => { value = (e.currentTarget as HTMLInputElement).value || undefined; onChange(); }}
           class="input-base" />

  {:else}
    <textarea bind:value={textBuffer} rows="8"
              oninput={syncFromText}
              class="input-base font-mono text-xs"
              placeholder={"{}"}></textarea>
    {#if parseError}
      <p class="text-xs" style="color: var(--status-error-text)">{parseError}</p>
    {/if}
  {/if}
</div>

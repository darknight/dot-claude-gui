<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onPatch,
  }: {
    settings: Settings;
    onPatch: (partial: Partial<Settings>) => void;
    error: string | null;
  } = $props();

  const entries = $derived(Object.entries(settings.env ?? {}));

  let newKey = $state("");
  let newValue = $state("");

  function update(key: string, value: string) {
    const next = { ...(settings.env ?? {}), [key]: value };
    onPatch({ env: next });
  }

  function remove(key: string) {
    const next = { ...(settings.env ?? {}) };
    delete next[key];
    onPatch({ env: Object.keys(next).length === 0 ? undefined : next });
  }

  function addNew() {
    if (!newKey || (settings.env ?? {})[newKey] != null) return;
    update(newKey, newValue);
    newKey = "";
    newValue = "";
  }
</script>

<div class="env-editor">
  <table>
    <thead>
      <tr><th>name</th><th>value</th><th></th></tr>
    </thead>
    <tbody>
      {#each entries as [key, value] (key)}
        <tr>
          <td><code>{key}</code></td>
          <td>
            <input
              type="text"
              value={value}
              oninput={(e) => update(key, (e.target as HTMLInputElement).value)}
            />
          </td>
          <td>
            <button type="button" onclick={() => remove(key)}>
              {t("common.remove")}
            </button>
          </td>
        </tr>
      {/each}
      <tr class="add-row">
        <td><input type="text" placeholder="NAME" bind:value={newKey} /></td>
        <td><input type="text" placeholder="value" bind:value={newValue} /></td>
        <td>
          <button type="button" onclick={addNew} disabled={!newKey}>
            {t("common.add")}
          </button>
        </td>
      </tr>
    </tbody>
  </table>
</div>

<style>
  .env-editor { max-width: 720px; }
  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; color: var(--text-muted); font-weight: 500; font-size: 12px; padding: 4px 8px; }
  td { padding: 4px 8px; }
  input {
    width: 100%;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
  }
  code { font-family: ui-monospace, Menlo, monospace; font-size: 13px; }
  button {
    padding: 2px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    cursor: pointer;
    color: inherit;
  }
  button[disabled] { opacity: 0.5; cursor: not-allowed; }
  .add-row { border-top: 1px solid var(--border); }
</style>

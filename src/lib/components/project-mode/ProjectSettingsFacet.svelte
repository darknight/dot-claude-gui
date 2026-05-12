<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { Settings } from "$lib/api/types";

  let { path }: { path: string } = $props();

  let raw = $state("");
  let original = $state("");
  let dirty = $derived(raw !== original);
  let error = $state<string | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  async function load() {
    loading = true;
    error = null;
    try {
      const resp = await ipcClient.projectReadSettings(path);
      raw = JSON.stringify(resp.settings ?? {}, null, 2);
      original = raw;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function validate(): Settings | null {
    try {
      const parsed = JSON.parse(raw);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        error = t("projectMode.settings.notObject");
        return null;
      }
      error = null;
      return parsed as Settings;
    } catch (e) {
      error = (e as Error).message;
      return null;
    }
  }

  async function save() {
    const parsed = validate();
    if (!parsed) return;
    saving = true;
    try {
      await ipcClient.projectWriteSettings(path, parsed);
      original = raw;
      toastStore.success(t("projectMode.settings.saved"));
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }

  function revert() {
    raw = original;
    error = null;
  }
</script>

<section class="settings-facet">
  <h2>{t("projectMode.settings.title")}</h2>
  <p class="hint">{t("projectMode.settings.hint", { path: `${path}/.claude/settings.json` })}</p>

  {#if loading}
    <div class="empty">{t("projectMode.settings.loading")}</div>
  {:else}
    <textarea
      bind:value={raw}
      onblur={validate}
      spellcheck="false"
      aria-label={t("projectMode.settings.title")}
    ></textarea>
    {#if error}
      <p class="err">{error}</p>
    {/if}
    <div class="actions">
      <button
        type="button"
        onclick={save}
        disabled={!dirty || saving || error !== null}
        class="primary"
      >{t("projectMode.settings.saveBtn")}</button>
      <button type="button" onclick={revert} disabled={!dirty}>
        {t("projectMode.settings.revertBtn")}
      </button>
    </div>
  {/if}
</section>

<style>
  .settings-facet {
    padding: 16px;
    height: 100%;
    display: flex;
    flex-direction: column;
    color: var(--text-primary);
  }
  h2 {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
  }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 0 0 12px;
  }
  textarea {
    flex: 1;
    min-height: 300px;
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
    line-height: 1.5;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    resize: vertical;
  }
  .err {
    color: var(--danger, #c44);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    margin: 8px 0 0;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  button {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button[disabled] {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button:hover:not([disabled]) {
    background: var(--bg-hover, rgba(0,0,0,0.05));
  }
  button.primary {
    background: var(--accent, #2c6cff);
    border-color: var(--accent, #2c6cff);
    color: white;
  }
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
</style>

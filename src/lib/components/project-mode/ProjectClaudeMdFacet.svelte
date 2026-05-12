<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";

  let { path }: { path: string } = $props();

  let content = $state("");
  let original = $state("");
  let dirty = $derived(content !== original);
  let loading = $state(true);
  let saving = $state(false);

  async function load() {
    loading = true;
    try {
      const resp = await ipcClient.projectReadClaudeMd(path);
      content = resp.content;
      original = content;
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  async function save() {
    saving = true;
    try {
      await ipcClient.projectWriteClaudeMd(path, content);
      original = content;
      toastStore.success(t("projectMode.claudemd.saved"));
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }

  function revert() {
    content = original;
  }
</script>

<section class="claudemd-facet">
  <h2>{t("projectMode.claudemd.title")}</h2>
  <p class="hint">{t("projectMode.claudemd.hint", { path: `${path}/.claude/CLAUDE.md` })}</p>

  {#if loading}
    <div class="empty">{t("projectMode.claudemd.loading")}</div>
  {:else}
    <textarea
      bind:value={content}
      spellcheck="false"
      aria-label={t("projectMode.claudemd.title")}
    ></textarea>
    <div class="actions">
      <button type="button" onclick={save} disabled={!dirty || saving} class="primary">
        {t("projectMode.claudemd.saveBtn")}
      </button>
      <button type="button" onclick={revert} disabled={!dirty}>
        {t("projectMode.claudemd.revertBtn")}
      </button>
    </div>
  {/if}
</section>

<style>
  .claudemd-facet {
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
  button[disabled] { opacity: 0.5; cursor: not-allowed; }
  button:hover:not([disabled]) { background: var(--bg-hover, rgba(0,0,0,0.05)); }
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

<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { MemoryFile } from "$lib/api/types";

  let { path }: { path: string } = $props();

  let files = $state<MemoryFile[]>([]);
  let memoryDir = $state("");
  let selected = $state<string | null>(null);
  let content = $state("");
  let original = $state("");
  let loadingList = $state(true);
  let loadingFile = $state(false);
  let saving = $state(false);
  let dirty = $derived(selected !== null && content !== original);

  async function loadList() {
    loadingList = true;
    try {
      const resp = await ipcClient.projectListMemory(path);
      files = resp.files;
      memoryDir = resp.path;
      if (selected && !files.some((f) => f.filename === selected)) {
        selected = null;
        content = "";
        original = "";
      }
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      loadingList = false;
    }
  }

  $effect(() => { void path; loadList(); });

  async function openFile(name: string) {
    if (dirty && !confirm(t("projectMode.memory.discardUnsaved"))) return;
    selected = name;
    loadingFile = true;
    try {
      content = await ipcClient.projectReadMemoryFile(path, name);
      original = content;
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      loadingFile = false;
    }
  }

  async function save() {
    if (!selected) return;
    saving = true;
    try {
      await ipcClient.projectWriteMemoryFile(path, selected, content);
      original = content;
      toastStore.success(t("projectMode.memory.saved"));
      await loadList();
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }

  async function deleteFile(name: string) {
    if (!confirm(t("projectMode.memory.confirmDelete", { name }))) return;
    try {
      await ipcClient.projectDeleteMemoryFile(path, name);
      if (selected === name) {
        selected = null;
        content = "";
        original = "";
      }
      await loadList();
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  async function newFile() {
    const name = prompt(t("projectMode.memory.newFilePrompt"));
    if (!name) return;
    const safe = name.endsWith(".md") ? name : `${name}.md`;
    try {
      await ipcClient.projectWriteMemoryFile(path, safe, "");
      await loadList();
      void openFile(safe);
    } catch (e) {
      toastStore.error(String(e));
    }
  }
</script>

<section class="memory-facet">
  <aside class="list-pane">
    <header>
      <h3>{t("projectMode.memory.title")}</h3>
      <button type="button" class="new-btn" onclick={newFile} aria-label={t("projectMode.memory.newFilePrompt")}>+</button>
    </header>
    {#if loadingList}
      <div class="empty">{t("projectMode.memory.loading")}</div>
    {:else if files.length === 0}
      <div class="empty">{t("projectMode.memory.noFiles")}</div>
    {:else}
      <ul>
        {#each files as f (f.filename)}
          <li class:active={selected === f.filename}>
            <button type="button" class="open" onclick={() => openFile(f.filename)}>
              <span class="name">{f.filename}</span>
              {#if f.description}<span class="desc">{f.description}</span>{/if}
            </button>
            <button type="button" class="del" onclick={() => deleteFile(f.filename)} aria-label={t("projectMode.memory.deleteAria")}>×</button>
          </li>
        {/each}
      </ul>
    {/if}
    <footer class="dir-path" title={memoryDir}>{memoryDir}</footer>
  </aside>

  <main class="viewer">
    {#if !selected}
      <div class="empty">{t("projectMode.memory.selectFile")}</div>
    {:else if loadingFile}
      <div class="empty">{t("projectMode.memory.loading")}</div>
    {:else}
      <textarea bind:value={content} spellcheck="false" aria-label={selected}></textarea>
      <div class="actions">
        <button type="button" onclick={save} disabled={!dirty || saving} class="primary">
          {t("projectMode.memory.saveBtn")}
        </button>
      </div>
    {/if}
  </main>
</section>

<style>
  .memory-facet {
    display: flex;
    height: 100%;
    color: var(--text-primary);
  }
  .list-pane {
    width: 260px;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .list-pane header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }
  .list-pane h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }
  .new-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    flex: 1;
    overflow: auto;
  }
  li {
    display: flex;
    align-items: stretch;
    border-bottom: 1px solid var(--border);
  }
  li.active {
    background: var(--bg-list-active, rgba(44, 108, 255, 0.1));
  }
  li button.open {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    background: transparent;
    border: none;
    padding: 8px 12px;
    color: inherit;
    cursor: pointer;
    text-align: left;
    gap: 2px;
  }
  li button.open:hover {
    background: var(--bg-hover, rgba(0, 0, 0, 0.05));
  }
  li .name {
    font-size: 13px;
  }
  li .desc {
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  li .del {
    background: transparent;
    border: none;
    padding: 0 8px;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    align-self: center;
  }
  li:hover .del {
    opacity: 1;
  }
  .dir-path {
    padding: 8px 12px;
    font-size: 10px;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .viewer {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
    min-height: 0;
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
    color: inherit;
    resize: vertical;
  }
  .actions {
    margin-top: 12px;
  }
  .actions button.primary {
    padding: 4px 12px;
    border: 1px solid var(--accent, #2c6cff);
    border-radius: 4px;
    background: var(--accent, #2c6cff);
    color: white;
    cursor: pointer;
  }
  .actions button[disabled] {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
</style>

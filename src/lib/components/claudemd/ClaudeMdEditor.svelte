<script lang="ts">
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import { t } from "$lib/i18n";

  let localContent = $state("");
  let originalContent = $state("");

  $effect(() => {
    const file = claudeMdStore.activeFile;
    if (file) {
      localContent = file.content;
      originalContent = file.content;
    } else {
      localContent = "";
      originalContent = "";
    }
  });

  let isDirty = $derived(localContent !== originalContent);

  $effect(() => {
    claudeMdStore.activeFileDirty = isDirty;
  });

  async function handleSave() {
    const file = claudeMdStore.activeFile;
    if (!file) return;
    await claudeMdStore.saveFile(file.id, localContent);
    originalContent = localContent;
  }

  async function handleDelete() {
    const file = claudeMdStore.activeFile;
    if (!file) return;
    if (!confirm(t("claudemd.confirmDelete"))) return;
    await claudeMdStore.deleteFile(file.id);
  }

  function scopeBadgeClass(scope: string): string {
    return scope === "global" ? "badge badge-info" : "badge badge-purple";
  }

  function scopeLabel(scope: string): string {
    return scope === "global" ? t("claudemd.scopeGlobal") : t("claudemd.scopeProject");
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !claudeMdStore.activeFile}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">{t("claudemd.selectFile")}</p>
    </div>
  {:else}
    {@const file = claudeMdStore.activeFile}
    <div class="border-b px-6 py-4" style="border-color: var(--border-color)">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="truncate text-sm font-semibold" style="color: var(--text-primary)">
              {file.filename}
            </h2>
            <span class="flex-shrink-0 {scopeBadgeClass(file.scope)}">
              {scopeLabel(file.scope)}
            </span>
            {#if isDirty}
              <span class="badge badge-warning flex-shrink-0">
                {t("common.unsaved")}
              </span>
            {/if}
          </div>
          <p class="mt-0.5 font-mono text-xs" style="color: var(--text-muted)">{file.path}</p>
        </div>

        <div class="flex flex-shrink-0 items-center gap-2">
          <button
            class="btn-primary rounded px-3 py-1.5 text-xs font-medium"
            disabled={!isDirty || claudeMdStore.saving}
            onclick={handleSave}
          >
            {claudeMdStore.saving ? t("common.saving") : t("common.save")}
          </button>
          {#if file.scope !== "global" || originalContent !== ""}
            <button
              class="btn-danger-ghost rounded px-3 py-1.5 text-xs font-medium"
              onclick={handleDelete}
            >
              {t("common.delete")}
            </button>
          {/if}
        </div>
      </div>
    </div>

    <div class="flex flex-1 flex-col overflow-hidden p-4">
      {#if claudeMdStore.loading}
        <div class="flex flex-1 items-center justify-center">
          <p class="text-sm" style="color: var(--text-muted)">{t("common.loading")}</p>
        </div>
      {:else}
        <textarea
          class="flex-1 resize-none font-mono text-xs leading-relaxed focus:outline-none"
          style="background-color: var(--bg-code); color: var(--text-primary); border: 1px solid var(--border-color); border-radius: 0.25rem; padding: 0.75rem"
          bind:value={localContent}
          spellcheck={false}
        ></textarea>
      {/if}
    </div>

    {#if claudeMdStore.error}
      <div class="border-t px-6 py-2 text-xs" style="color: var(--status-error-text); border-color: var(--border-color)">
        {claudeMdStore.error}
      </div>
    {/if}
  {/if}
</div>

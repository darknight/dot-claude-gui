<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import type { Settings } from "$lib/api/types";
  import SectionedSettings from "$lib/components/shared/SectionedSettings.svelte";
  import ProjectRuntimeEditor from "./settings/ProjectRuntimeEditor.svelte";
  import ProjectEnvVarEditor from "./settings/ProjectEnvVarEditor.svelte";
  import ProjectHooksEditor from "./settings/ProjectHooksEditor.svelte";
  import ProjectAdvancedJsonEditor from "./settings/ProjectAdvancedJsonEditor.svelte";

  let { path }: { path: string } = $props();

  let original = $state<Settings>({});
  let current = $state<Settings>({});
  let activeSection = $state("runtime");
  let error = $state<string | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  let isDirty = $derived(JSON.stringify(current) !== JSON.stringify(original));

  const sections = $derived([
    { id: "runtime",     label: t("projectMode.settings.section.runtime") },
    { id: "environment", label: t("projectMode.settings.section.environment") },
    { id: "hooks",       label: t("projectMode.settings.section.hooks") },
    { id: "advanced",    label: t("projectMode.settings.section.advanced") },
  ]);

  async function load() {
    loading = true;
    error = null;
    try {
      const resp = await ipcClient.projectReadSettings(path);
      original = (resp.settings ?? {}) as Settings;
      current = JSON.parse(JSON.stringify(original));
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function onPatch(partial: Partial<Settings>) {
    current = { ...current, ...partial };
  }

  function onReplace(next: Partial<Settings>) {
    // Used by AdvancedJsonEditor to canonicalize the entire draft.
    current = next as Settings;
  }

  async function save() {
    saving = true;
    try {
      await ipcClient.projectWriteSettings(path, current);
      original = JSON.parse(JSON.stringify(current));
      toastStore.success(t("projectMode.settings.saved"));
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }

  function revert() {
    current = JSON.parse(JSON.stringify(original));
    error = null;
  }
</script>

<section class="settings-facet">
  <header>
    <h2>{t("projectMode.settings.title")}</h2>
    <p class="hint">{t("projectMode.settings.hint", { path: `${path}/.claude/settings.json` })}</p>
  </header>

  {#if loading}
    <div class="empty">{t("projectMode.settings.loading")}</div>
  {:else}
    <SectionedSettings {sections} bind:activeSection {isDirty} {error}>
      {#snippet content(section)}
        {#if section === "runtime"}
          <ProjectRuntimeEditor settings={current} onPatch={onPatch} {error} />
        {:else if section === "environment"}
          <ProjectEnvVarEditor settings={current} onPatch={onPatch} {error} />
        {:else if section === "hooks"}
          <ProjectHooksEditor settings={current} onPatch={onPatch} {error} />
        {:else}
          <ProjectAdvancedJsonEditor settings={current} {onReplace} {error} />
        {/if}
      {/snippet}
    </SectionedSettings>

    <div class="actions">
      <button
        type="button"
        onclick={save}
        disabled={!isDirty || saving || error !== null}
        class="primary"
      >{t("projectMode.settings.saveBtn")}</button>
      <button type="button" onclick={revert} disabled={!isDirty}>
        {t("projectMode.settings.revertBtn")}
      </button>
    </div>
  {/if}
</section>

<style>
  .settings-facet {
    padding: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    color: var(--text-primary);
  }
  header {
    padding: 16px 16px 8px;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
  }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 0 0 8px;
  }
  .actions {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border, transparent);
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

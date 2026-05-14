<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onReplace,
    error,
  }: {
    settings: Settings;
    onReplace: (next: Partial<Settings>) => void;
    error: string | null;
  } = $props();

  let raw = $state(JSON.stringify(settings, null, 2));
  let lastSettingsKey = $state("");

  $effect(() => {
    const key = JSON.stringify(settings);
    if (key !== lastSettingsKey) {
      raw = JSON.stringify(settings, null, 2);
      lastSettingsKey = key;
    }
  });

  let localError = $state<string | null>(null);

  function onChange(e: Event) {
    raw = (e.target as HTMLTextAreaElement).value;
    try {
      const parsed = JSON.parse(raw);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        localError = t("projectMode.settings.notObject");
        return;
      }
      localError = null;
      onReplace(parsed as Partial<Settings>);
    } catch (e) {
      localError = (e as Error).message;
    }
  }
</script>

<div class="advanced">
  <textarea
    value={raw}
    oninput={onChange}
    spellcheck="false"
    aria-label={t("projectMode.settings.section.advanced")}
  ></textarea>
  {#if localError || error}
    <p class="err">{localError ?? error}</p>
  {/if}
</div>

<style>
  .advanced {
    display: flex;
    flex-direction: column;
    height: 100%;
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
</style>

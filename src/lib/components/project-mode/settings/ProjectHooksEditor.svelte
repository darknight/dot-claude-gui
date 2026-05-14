<script lang="ts">
  import { t } from "$lib/i18n";
  import type { Settings } from "$lib/api/types";

  let {
    settings,
    onPatch,
    onError,
  }: {
    settings: Settings;
    onPatch: (partial: Partial<Settings>) => void;
    error: string | null;
    onError?: (err: string | null) => void;
  } = $props();

  let raw = $state(JSON.stringify(settings.hooks ?? {}, null, 2));
  let lastSettingsKey = $state("");
  let localError = $state<string | null>(null);

  $effect(() => { onError?.(localError); });

  $effect(() => {
    const key = JSON.stringify(settings.hooks);
    if (key !== lastSettingsKey) {
      raw = JSON.stringify(settings.hooks ?? {}, null, 2);
      lastSettingsKey = key;
    }
  });

  function onChange(e: Event) {
    raw = (e.target as HTMLTextAreaElement).value;
    try {
      const parsed = JSON.parse(raw);
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        localError = t("projectMode.settings.notObject");
        return;
      }
      localError = null;
      onPatch({ hooks: Object.keys(parsed).length === 0 ? undefined : parsed as Settings["hooks"] });
    } catch (e) {
      localError = (e as Error).message;
    }
  }
</script>

<div class="hooks">
  <p class="hint">{t("projectMode.settings.section.hooksHint")}</p>
  <textarea
    value={raw}
    oninput={onChange}
    spellcheck="false"
    aria-label={t("projectMode.settings.section.hooks")}
  ></textarea>
  {#if localError}<p class="err">{localError}</p>{/if}
</div>

<style>
  .hooks { display: flex; flex-direction: column; height: 100%; }
  .hint { color: var(--text-muted); font-size: 12px; margin: 0 0 8px; }
  textarea {
    flex: 1;
    min-height: 240px;
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

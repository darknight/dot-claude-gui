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

  function setModel(e: Event) {
    const value = (e.target as HTMLInputElement).value.trim();
    onPatch({ model: value === "" ? undefined : value });
  }

  function setOutputStyle(e: Event) {
    const value = (e.target as HTMLInputElement).value.trim();
    onPatch({ outputStyle: value === "" ? undefined : value });
  }
</script>

<div class="runtime-fields">
  <label>
    <span>model</span>
    <input
      type="text"
      value={settings.model ?? ""}
      oninput={setModel}
      placeholder="claude-opus-4-7"
    />
  </label>
  <label>
    <span>outputStyle</span>
    <input
      type="text"
      value={settings.outputStyle ?? ""}
      oninput={setOutputStyle}
      placeholder=""
    />
  </label>
  <p class="hint">{t("projectMode.settings.section.runtimeHint")}</p>
</div>

<style>
  .runtime-fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 480px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label span {
    color: var(--text-muted);
    font-size: 13px;
  }
  input {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: var(--text-primary);
    font-family: ui-monospace, Menlo, monospace;
    font-size: 13px;
  }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 4px 0 0;
  }
</style>

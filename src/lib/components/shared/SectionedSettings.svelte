<script lang="ts">
  import type { Snippet } from "svelte";
  import { t } from "$lib/i18n";

  type Section = { id: string; label: string };

  let {
    sections,
    activeSection,
    onChange,
    isDirty,
    error,
    content,
  }: {
    sections: Section[];
    activeSection: string;
    onChange: (id: string) => void;
    isDirty: boolean;
    error: string | null;
    content: Snippet<[string]>;
  } = $props();
</script>

<div class="sectioned">
  <nav class="section-nav" aria-label="Sections">
    {#each sections as section (section.id)}
      <button
        type="button"
        class="section-link"
        class:active={section.id === activeSection}
        onclick={() => onChange(section.id)}
      >{section.label}</button>
    {/each}
  </nav>

  <div class="section-body">
    {#if isDirty}
      <div class="dirty-bar">
        <span>{t("common.unsavedChanges")}</span>
      </div>
    {/if}
    {#if error}
      <div class="error-bar">{error}</div>
    {/if}
    <div class="section-content">
      {@render content(activeSection)}
    </div>
  </div>
</div>

<style>
  .sectioned {
    display: grid;
    grid-template-columns: 200px 1fr;
    height: 100%;
    min-height: 0;
  }
  .section-nav {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border, transparent);
    background: var(--bg-secondary, transparent);
    padding: 8px 0;
    overflow-y: auto;
  }
  .section-link {
    text-align: left;
    padding: 6px 16px;
    background: transparent;
    border: 0;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 13px;
  }
  .section-link.active {
    background: var(--accent-bg, rgba(44,108,255,0.12));
    color: var(--accent-text, inherit);
    font-weight: 600;
  }
  .section-link:hover:not(.active) {
    background: var(--bg-hover, rgba(0,0,0,0.04));
  }
  .section-body {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }
  .dirty-bar {
    border-bottom: 1px solid var(--border, transparent);
    background: var(--bg-secondary, transparent);
    padding: 6px 16px;
    font-size: 12px;
    color: var(--status-warning-text, inherit);
  }
  .error-bar {
    border-bottom: 1px solid var(--status-error-text, transparent);
    background: var(--status-error-bg, rgba(196,68,68,0.08));
    color: var(--status-error-text, inherit);
    padding: 6px 16px;
    font-size: 12px;
  }
  .section-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }
</style>

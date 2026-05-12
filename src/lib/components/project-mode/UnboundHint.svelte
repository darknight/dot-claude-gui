<script lang="ts">
  import { t } from "$lib/i18n";
  import { modeStore } from "$lib/stores/mode.svelte";
</script>

<div class="empty">
  <p>{t("projectMode.unboundHint")}</p>
  <button onclick={() => modeStore.setSelectedProjectFacet("binding")}>
    {t("projectMode.goToBinding")}
  </button>
</div>

<style>
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-muted);
  }
  .empty button {
    margin-top: 8px;
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
</style>

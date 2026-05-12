<script lang="ts">
  import { t } from "$lib/i18n";
  import { projectsStore } from "$lib/stores/projects.svelte";

  let { path }: { path: string } = $props();

  async function onRemove() {
    if (!confirm(t("projectMode.staleConfirmRemove"))) return;
    await projectsStore.remove(path);
  }
</script>

<div class="banner" role="alert">
  <span>{t("projectMode.staleBanner", { path })}</span>
  <button class="remove-btn" onclick={onRemove}>{t("projectMode.staleRemoveBtn")}</button>
</div>

<style>
  .banner {
    background: var(--bg-warn, #fde2e2);
    color: var(--text-warn, #8a1f1f);
    padding: 8px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .remove-btn {
    background: transparent;
    border: 1px solid currentColor;
    padding: 2px 10px;
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
  }
</style>

<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";

  let { path }: { path: string } = $props();

  async function onUpdate() {
    try {
      const parent = path.replace(/[\\/][^\\/]+[\\/]?$/, "") || undefined;
      const picked = await open({
        directory: true,
        multiple: false,
        title: t("projectMode.staleUpdatePathDialogTitle"),
        defaultPath: parent,
      });
      if (typeof picked !== "string") return; // cancelled
      await projectsStore.updatePath(path, picked);
      modeStore.setSelectedProject(picked);
      toastStore.success(t("projectMode.stalePathUpdated"));
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  async function onRemove() {
    if (!confirm(t("projectMode.staleConfirmRemove"))) return;
    await projectsStore.remove(path);
  }
</script>

<div class="banner" role="alert">
  <span>{t("projectMode.staleBanner", { path })}</span>
  <div class="actions">
    <button class="update-btn" onclick={onUpdate}>{t("projectMode.staleUpdatePathBtn")}</button>
    <button class="remove-btn" onclick={onRemove}>{t("projectMode.staleRemoveBtn")}</button>
  </div>
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
  .actions {
    display: flex;
    gap: 8px;
  }
  .update-btn,
  .remove-btn {
    background: transparent;
    border: 1px solid currentColor;
    padding: 2px 10px;
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
  }
  .update-btn {
    font-weight: 600;
  }
</style>

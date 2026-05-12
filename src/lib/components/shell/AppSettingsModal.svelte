<script lang="ts">
  import AppSettingsView from "$lib/components/appsettings/AppSettingsView.svelte";
  import { t } from "$lib/i18n";

  let { open = false, onClose } = $props<{ open: boolean; onClose: () => void }>();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={handleKey} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center"
    style="background-color: rgba(0, 0, 0, 0.5)"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    aria-label={t("shell.appSettings")}
  >
    <div
      class="w-[80vw] max-w-4xl h-[80vh] rounded-lg overflow-hidden flex flex-col"
      style="background-color: var(--bg-primary); border: 1px solid var(--border-color)"
    >
      <header class="flex items-center justify-between px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
        <h2 class="text-sm font-semibold" style="color: var(--text-primary)">{t("shell.appSettings")}</h2>
        <button
          class="p-1 rounded transition-colors hover:bg-[var(--bg-card-hover)]"
          style="color: var(--text-secondary)"
          onclick={onClose}
          aria-label={t("shell.close")}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
          </svg>
        </button>
      </header>
      <div class="flex-1 overflow-auto">
        <AppSettingsView />
      </div>
    </div>
  </div>
{/if}

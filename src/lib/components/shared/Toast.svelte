<script lang="ts">
  import { fly } from "svelte/transition";
  import { toastStore } from "$lib/stores/toast.svelte";

  const iconPaths: Record<string, string> = {
    success:
      "M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
    error:
      "M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z",
    warning:
      "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.008v.008H12v-.008Z",
    info: "M11.25 11.25l.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z",
  };

  const borderColors: Record<string, string> = {
    success: "var(--status-success-text)",
    error: "var(--status-error-text)",
    warning: "var(--status-warning-text)",
    info: "var(--status-info-text)",
  };

  const iconColors: Record<string, string> = {
    success: "#3fb950",
    error: "#f85149",
    warning: "#d29922",
    info: "#58a6ff",
  };
</script>

{#if toastStore.toasts.length > 0}
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2" style="max-width: 360px;">
    {#each toastStore.toasts as toast (toast.id)}
      <div
        class="flex items-center gap-3 rounded-lg px-4 py-3 shadow-lg"
        style="background-color: var(--bg-secondary); border: 1px solid {borderColors[toast.type]};"
        transition:fly={{ x: 100, duration: 300 }}
      >
        <svg
          class="h-5 w-5 flex-shrink-0"
          fill="none"
          stroke={iconColors[toast.type]}
          viewBox="0 0 24 24"
          stroke-width="1.5"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d={iconPaths[toast.type]}
          />
        </svg>
        <span class="flex-1 text-sm" style="color: var(--text-primary);">
          {toast.message}
        </span>
        <button
          class="flex-shrink-0 text-lg leading-none opacity-50 transition-opacity hover:opacity-100"
          style="color: var(--text-secondary);"
          onclick={() => toastStore.dismiss(toast.id)}
        >
          &times;
        </button>
      </div>
    {/each}
  </div>
{/if}

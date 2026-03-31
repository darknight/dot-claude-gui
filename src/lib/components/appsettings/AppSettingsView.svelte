<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
  import ConnectionsPanel from "./ConnectionsPanel.svelte";

  let { activeSub = "appearance" }: { activeSub?: string } = $props();
</script>

<div class="p-6 space-y-8">
  {#if activeSub === "connections"}
    <ConnectionsPanel />
  {:else}
    <section class="space-y-4">
      <h2 class="text-lg font-medium text-gray-100">外观</h2>

      <div>
        <label class="block text-sm text-gray-400 mb-1">主题</label>
        <select
          class="bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
          value={appSettingsStore.preferences.theme}
          onchange={(e) => appSettingsStore.update({ theme: (e.target as HTMLSelectElement).value as "light" | "dark" | "system" })}
        >
          <option value="system">跟随系统</option>
          <option value="dark">深色</option>
          <option value="light">浅色</option>
        </select>
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1">字体大小: {appSettingsStore.preferences.fontSize}px</label>
        <input
          type="range"
          min="12"
          max="20"
          value={appSettingsStore.preferences.fontSize}
          class="w-48"
          oninput={(e) => appSettingsStore.update({ fontSize: parseInt((e.target as HTMLInputElement).value) })}
        />
      </div>
    </section>
  {/if}
</div>

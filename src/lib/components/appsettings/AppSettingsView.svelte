<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
</script>

<div class="p-6 space-y-8">
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">外观</h2>

    <div>
      <label class="block text-sm mb-1" style="color: var(--text-muted)">主题</label>
      <select
        class="input-base w-auto"
        value={appSettingsStore.preferences.theme}
        onchange={(e) => appSettingsStore.update({ theme: (e.target as HTMLSelectElement).value as "light" | "dark" | "system" })}
      >
        <option value="system">跟随系统</option>
        <option value="dark">深色</option>
        <option value="light">浅色</option>
      </select>
    </div>

    <div>
      <label class="block text-sm mb-1" style="color: var(--text-muted)">字体大小: {appSettingsStore.preferences.fontSize}px</label>
      <input
        type="range"
        min="12"
        max="20"
        value={appSettingsStore.preferences.fontSize}
        class="w-48"
        oninput={(e) => appSettingsStore.update({ fontSize: parseInt((e.target as HTMLInputElement).value) })}
      />
    </div>

    <div>
      <label class="block text-sm mb-1" style="color: var(--text-muted)">语言</label>
      <select
        class="input-base w-auto"
        value={appSettingsStore.preferences.language}
        onchange={(e) => appSettingsStore.update({ language: (e.target as HTMLSelectElement).value })}
      >
        <option value="zh-CN">简体中文</option>
        <option value="en-US">English</option>
      </select>
    </div>
  </section>
</div>

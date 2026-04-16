<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
  import { t, ACTIVE_LOCALES, localeDisplayName, type Locale } from "$lib/i18n";
</script>

<div class="p-6 space-y-8">
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.appearance")}</h2>

    <div>
      <label class="block text-sm mb-1" style="color: var(--text-muted)">{t("appsettings.theme")}</label>
      <select
        class="input-base w-auto"
        value={appSettingsStore.preferences.theme}
        onchange={(e) => appSettingsStore.update({ theme: (e.target as HTMLSelectElement).value as "light" | "dark" | "system" })}
      >
        <option value="system">{t("appsettings.themeSystem")}</option>
        <option value="dark">{t("appsettings.themeDark")}</option>
        <option value="light">{t("appsettings.themeLight")}</option>
      </select>
    </div>

    <div>
      <label class="block text-sm mb-1" style="color: var(--text-muted)">{t("appsettings.fontSize", { size: appSettingsStore.preferences.fontSize })}</label>
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
      <label class="block text-sm mb-1" style="color: var(--text-muted)">{t("appsettings.languageLabel")}</label>
      <select
        class="input-base w-auto"
        value={appSettingsStore.preferences.language}
        onchange={(e) => appSettingsStore.update({ language: (e.target as HTMLSelectElement).value as Locale })}
      >
        {#each ACTIVE_LOCALES as loc}
          <option value={loc}>{localeDisplayName(loc)}</option>
        {/each}
      </select>
    </div>
  </section>
</div>

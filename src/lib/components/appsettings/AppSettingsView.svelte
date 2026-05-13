<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
  import { t, ACTIVE_LOCALES, localeDisplayName, type Locale } from "$lib/i18n";
  import pkg from "../../../../package.json" with { type: "json" };

  const APP_VERSION = (pkg as { version: string }).version;
  const REPO_URL = (pkg as { repository: { url: string } }).repository.url
    .replace(/^git\+/, "")
    .replace(/\.git$/, "");
  const APP_NAME = (pkg as { name: string }).name;
</script>

<div class="p-6 space-y-8">

  <!-- 1. Appearance -->
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
  </section>

  <!-- 2. Language -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.language")}</h2>

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

  <!-- 3. Terminal -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.terminal")}</h2>

    <div>
      <label class="block text-sm mb-1" style="color: var(--text-muted)">{t("appsettings.preferredTerminal")}</label>
      <select
        class="input-base w-auto"
        value={appSettingsStore.preferences.preferredTerminal ?? "terminal"}
        onchange={(e) => appSettingsStore.update({ preferredTerminal: (e.target as HTMLSelectElement).value as "terminal" | "iterm2" })}
      >
        <option value="terminal">{t("appsettings.terminalDefault")}</option>
        <option value="iterm2">{t("appsettings.terminalIterm2")}</option>
      </select>
    </div>
  </section>

  <!-- 4. About -->
  <section class="space-y-4">
    <h2 class="text-lg font-medium" style="color: var(--text-primary)">{t("appsettings.about")}</h2>
    <div class="space-y-1 text-sm" style="color: var(--text-muted)">
      <div>{APP_NAME}</div>
      <div>{t("appsettings.version", { version: APP_VERSION })}</div>
      <div>
        <span>{t("appsettings.repo")}: </span>
        <a href={REPO_URL} target="_blank" rel="noreferrer" style="color: var(--accent-primary)">{REPO_URL}</a>
      </div>
    </div>
  </section>

</div>

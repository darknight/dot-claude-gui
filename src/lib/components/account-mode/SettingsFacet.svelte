<script lang="ts">
  import SettingsEditor from "$lib/components/settings/SettingsEditor.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  const sections = [
    { id: "general", labelKey: "settings.general" },
    { id: "permissions", labelKey: "settings.permissions" },
    { id: "hooks", labelKey: "settings.hooks" },
    { id: "sandbox", labelKey: "settings.sandbox" },
    { id: "environment", labelKey: "settings.environment" },
    { id: "statusline", labelKey: "settings.statusLine" },
    { id: "runtime", labelKey: "settings.runtime" },
    { id: "mcpPolicy", labelKey: "settings.mcpPolicy" },
    { id: "pluginsMarketplace", labelKey: "settings.pluginsMarketplace" },
    { id: "advanced", labelKey: "settings.advanced" },
  ] satisfies { id: string; labelKey: MessageKey }[];

  let active = $state("general");
</script>

<div class="flex flex-col flex-1 overflow-hidden">
  <nav
    class="flex items-center gap-1 px-2 py-1 overflow-x-auto"
    style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
  >
    {#each sections as section (section.id)}
      <button
        type="button"
        class="px-2.5 py-1 text-xs rounded transition-colors whitespace-nowrap {active === section.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
        style="background-color: {active === section.id ? 'var(--accent-bg)' : 'transparent'}; color: {active === section.id ? 'var(--accent-text)' : 'var(--text-secondary)'}"
        onclick={() => { active = section.id; }}
      >
        {t(section.labelKey)}
      </button>
    {/each}
  </nav>

  <div class="flex-1 overflow-hidden">
    {#if configStore.loading}
      <div class="p-6">
        <p class="text-sm" style="color: var(--text-muted)">{t("nav.loadingConfig")}</p>
      </div>
    {:else}
      <SettingsEditor activeSection={active} />
    {/if}
  </div>
</div>

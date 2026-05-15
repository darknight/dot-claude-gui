<script lang="ts">
  import { modeStore } from "$lib/stores/mode.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t, type MessageKey } from "$lib/i18n";

  import Overview from "./Overview.svelte";
  import SettingsFacet from "./SettingsFacet.svelte";
  import PluginsFacet from "./PluginsFacet.svelte";
  import SkillsFacet from "./SkillsFacet.svelte";
  import ClaudeMdFacet from "./ClaudeMdFacet.svelte";
  import MemoryFacet from "./MemoryFacet.svelte";
  import McpFacet from "./McpFacet.svelte";

  type Facet = "overview" | "settings" | "plugins" | "skills" | "claudemd" | "memory" | "mcp";

  const facets = [
    { id: "overview", labelKey: "accountMode.overview" },
    { id: "settings", labelKey: "accountMode.settings" },
    { id: "plugins", labelKey: "accountMode.plugins" },
    { id: "skills", labelKey: "accountMode.skills" },
    { id: "claudemd", labelKey: "accountMode.claudemd" },
    { id: "memory", labelKey: "accountMode.memory" },
    { id: "mcp", labelKey: "accountMode.mcp" },
  ] satisfies { id: Facet; labelKey: MessageKey }[];

  const activeFacet = $derived<Facet>(
    modeStore.accountFacet(modeStore.selectedAccount) as Facet,
  );

  // Default-selection: if no account is selected and we have accounts, pick the first.
  $effect(() => {
    if (modeStore.selectedAccount === null && accountsStore.accounts.length > 0) {
      modeStore.setSelectedAccount(accountsStore.accounts[0].name);
    }
  });

  // When selectedAccount changes, switch the active account on the backend and
  // reload all caches that depend on the user-layer dir.
  $effect(() => {
    const name = modeStore.selectedAccount;
    if (!name) return;
    void (async () => {
      try {
        await ipcClient.setActiveAccount(name);
        await Promise.all([
          configStore.loadUserConfig(),
          pluginsStore.loadPlugins(),
          skillsStore.loadSkills(),
          memoryStore.loadProjects(),
          mcpStore.loadServers(),
          claudeMdStore.loadFiles(),
        ]);
      } catch (e) {
        toastStore.error(t("shell.switchAccountFailed"));
        console.error("account switch reload failed", e);
      }
    })();
  });
</script>

{#if modeStore.selectedAccount === null}
  <div class="flex-1 flex items-center justify-center p-6">
    <p class="text-sm" style="color: var(--text-muted)">{t("accountMode.selectAccountHint")}</p>
  </div>
{:else}
  <div class="flex flex-col flex-1 overflow-hidden">
    <!-- Facet tab strip -->
    <div
      class="flex items-center gap-0.5 px-2 pt-2"
      style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)"
    >
      {#each facets as f (f.id)}
        <button
          class="px-3 py-2 text-sm rounded-t-md transition-colors {activeFacet === f.id ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
          style="background-color: {activeFacet === f.id ? 'var(--bg-primary)' : 'transparent'}; color: {activeFacet === f.id ? 'var(--text-primary)' : 'var(--text-secondary)'}; border: 1px solid {activeFacet === f.id ? 'var(--border-color)' : 'transparent'}; border-bottom: none"
          onclick={() => {
            if (modeStore.selectedAccount) {
              modeStore.setAccountFacet(modeStore.selectedAccount, f.id);
            }
          }}
        >
          {t(f.labelKey)}
        </button>
      {/each}
    </div>

    <!-- Facet body -->
    <div class="flex-1 overflow-hidden flex flex-col">
      {#if activeFacet === "overview"}
        <Overview accountName={modeStore.selectedAccount} />
      {:else if activeFacet === "settings"}
        <SettingsFacet />
      {:else if activeFacet === "plugins"}
        <PluginsFacet />
      {:else if activeFacet === "skills"}
        <SkillsFacet />
      {:else if activeFacet === "claudemd"}
        <ClaudeMdFacet />
      {:else if activeFacet === "memory"}
        <MemoryFacet />
      {:else if activeFacet === "mcp"}
        <McpFacet />
      {/if}
    </div>
  </div>
{/if}

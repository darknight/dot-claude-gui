<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { onConfigChanged } from "$lib/ipc/events.js";

  import TopBar from "$lib/components/shell/TopBar.svelte";
  import AppSettingsModal from "$lib/components/shell/AppSettingsModal.svelte";
  import AccountSidebar from "$lib/components/shell/AccountSidebar.svelte";
  import ProjectSidebar from "$lib/components/shell/ProjectSidebar.svelte";
  import AccountModeView from "$lib/components/account-mode/AccountModeView.svelte";
  import ProjectModeView from "$lib/components/project-mode/ProjectModeView.svelte";
  import ResizeHandle from "$lib/components/shared/ResizeHandle.svelte";
  import Toast from "$lib/components/shared/Toast.svelte";

  // ── Theme / font / lang effects ────────────────────────────────────
  $effect(() => {
    const theme = appSettingsStore.preferences.theme;
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else if (theme === "light") {
      document.documentElement.classList.remove("dark");
    } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  });

  $effect(() => {
    const lang = appSettingsStore.preferences.language;
    if (lang) document.documentElement.lang = lang;
  });

  $effect(() => {
    document.documentElement.style.setProperty(
      "--app-font-size",
      appSettingsStore.preferences.fontSize + "px"
    );
  });

  $effect(() => {
    document.documentElement.style.setProperty(
      "--sidebar-width",
      appSettingsStore.preferences.sidebarWidth + "px"
    );
  });

  let unlistenConfigChanged: (() => void) | undefined;
  let settingsModalOpen = $state(false);

  onMount(() => {
    void (async () => {
      await appSettingsStore.load();
      await Promise.all([
        configStore.loadUserConfig(),
        projectsStore.loadProjects(),
        pluginsStore.loadPlugins(),
        skillsStore.loadSkills(),
        memoryStore.loadProjects(),
        mcpStore.loadServers(),
        claudeMdStore.loadFiles(),
        accountsStore.loadAccounts(),
      ]);
      unlistenConfigChanged = await onConfigChanged((payload) => {
        configStore.setUserConfig(payload.settings);
      });
    })();

    return () => {
      unlistenConfigChanged?.();
    };
  });
</script>

{#if appSettingsStore.loaded}
  <div
    class="flex h-screen w-screen flex-col overflow-hidden"
    style="background-color: var(--bg-primary); color: var(--text-primary)"
  >
    <TopBar onOpenSettings={() => { settingsModalOpen = true; }} />

    <div class="flex flex-1 overflow-hidden">
      <!-- Sidebar (mode-aware) -->
      <aside
        class="flex-shrink-0 overflow-hidden"
        style="width: var(--sidebar-width); background-color: var(--bg-secondary); border-right: 1px solid var(--border-color); min-width: 200px"
      >
        {#if modeStore.mode === "account"}
          <AccountSidebar />
        {:else}
          <ProjectSidebar />
        {/if}
      </aside>

      <ResizeHandle
        min={200}
        max={400}
        onResize={(w) => appSettingsStore.update({ sidebarWidth: w })}
      />

      <!-- Main -->
      <main class="flex-1 flex flex-col overflow-hidden">
        {#if modeStore.mode === "account"}
          <AccountModeView />
        {:else}
          <ProjectModeView />
        {/if}
      </main>
    </div>

    <AppSettingsModal open={settingsModalOpen} onClose={() => { settingsModalOpen = false; }} />
    <Toast />
  </div>
{:else}
  <div class="flex h-screen w-screen items-center justify-center" style="background-color: var(--bg-primary)"></div>
{/if}

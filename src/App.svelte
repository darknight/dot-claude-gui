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
  import { ipcClient } from "$lib/ipc/client.js";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";

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
    const onWindowFocus = () => { void projectsStore.loadProjects(); };
    window.addEventListener("focus", onWindowFocus);

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
      modeStore.pruneStale(
        new Set(accountsStore.accounts.map((a) => a.name)),
        new Set(projectsStore.entries.map((p) => p.path)),
      );
      unlistenConfigChanged = await onConfigChanged((payload) => {
        configStore.setUserConfig(payload.settings);
      });
      // Pull the one-shot migration report (IPC pull avoids setup-vs-mount race).
      try {
        const report = await ipcClient.takeMigrationReport();
        if (report?.migrated) {
          toastStore.success(t("migration.toastSuccess", { backup: report.bakPath ?? "" }));
        }
      } catch (e) {
        // Migration IPC failure is non-fatal — don't break startup.
        console.warn("take_migration_report failed", e);
      }

      // Pre-flight: production .app launched from Finder inherits a minimal
      // PATH and won't find `claude` (commonly at /opt/homebrew/bin). Surface
      // this once with a sticky toast so plugin/mcp commands don't appear to
      // "silently do nothing" when the CLI can't be spawned.
      try {
        const status = await ipcClient.checkClaudeCli();
        if (!status.resolved) {
          toastStore.error(t("preflight.claudeNotFound"), 0);
        }
      } catch (e) {
        console.warn("check_claude_cli failed", e);
      }
    })();

    return () => {
      window.removeEventListener("focus", onWindowFocus);
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

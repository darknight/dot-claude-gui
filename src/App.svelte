<script lang="ts">
  import { onMount } from "svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { connectionsStore } from "$lib/stores/connections.svelte";
  import ConnectionStatus from "$lib/components/shared/ConnectionStatus.svelte";
  import EnvironmentSelector from "$lib/components/shared/EnvironmentSelector.svelte";
  import ScopeSelector from "$lib/components/shared/ScopeSelector.svelte";
  import SettingsEditor from "$lib/components/settings/SettingsEditor.svelte";
  import PluginsModule from "$lib/components/plugins/PluginsModule.svelte";
  import SkillsModule from "$lib/components/skills/SkillsModule.svelte";
  import MemoryList from "$lib/components/memory/MemoryList.svelte";
  import MemoryModule from "$lib/components/memory/MemoryModule.svelte";
  import McpModule from "$lib/components/mcp/McpModule.svelte";
  import EffectiveConfigView from "$lib/components/effective/EffectiveConfigView.svelte";
  import LauncherView from "$lib/components/launcher/LauncherView.svelte";
  import AppSettingsView from "$lib/components/appsettings/AppSettingsView.svelte";

  // ---------------------------------------------------------------------------
  // Theme effect
  // ---------------------------------------------------------------------------

  $effect(() => {
    const theme = appSettingsStore.preferences.theme;
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else if (theme === "light") {
      document.documentElement.classList.remove("dark");
    } else {
      // system
      if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }
    }
  });

  // ---------------------------------------------------------------------------
  // Navigation state
  // ---------------------------------------------------------------------------

  let activeNav = $state("S");

  const navButtons = [
    { id: "S", label: "Settings" },
    { id: "P", label: "Plugins" },
    { id: "K", label: "Skills" },
    { id: "M", label: "Memory" },
    { id: "C", label: "MCP Servers" },
    { id: "E", label: "Effective Config" },
    { id: "L", label: "Launcher" },
  ];

  // App Settings is kept separate (bottom of sidebar)
  const appSettingsButton = { id: "A", label: "App Settings" };

  // App Settings sub-navigation
  let appSettingsSub = $state("appearance");

  // ---------------------------------------------------------------------------
  // Settings sub-navigation
  // ---------------------------------------------------------------------------

  const settingsSections = [
    { id: "general", label: "General" },
    { id: "permissions", label: "Permissions" },
    { id: "hooks", label: "Hooks" },
    { id: "sandbox", label: "Sandbox" },
    { id: "environment", label: "Environment" },
    { id: "statusline", label: "Status Line" },
  ];

  let settingsSection = $state("general");

  // ---------------------------------------------------------------------------
  // Plugins sub-navigation
  // ---------------------------------------------------------------------------

  const pluginsSections = [
    { id: "installed", label: "Installed" },
    { id: "marketplace", label: "Marketplace" },
    { id: "manage-marketplaces", label: "Manage Marketplaces" },
    { id: "per-project", label: "Per-Project" },
  ];

  let pluginsSection = $state("installed");

  // ---------------------------------------------------------------------------
  // MCP sub-navigation
  // ---------------------------------------------------------------------------

  const mcpSections = [
    { id: "servers", label: "Servers" },
    { id: "add", label: "Add Server" },
  ];

  let mcpSection = $state("servers");

  // ---------------------------------------------------------------------------
  // Mount: load connections, connect to active daemon
  // ---------------------------------------------------------------------------

  onMount(() => {
    void (async () => {
      await appSettingsStore.load();
      await connectionsStore.load();

      const active = connectionsStore.activeConnection;
      if (active) {
        await connectionStore.connect(active.url, active.token);
      }
    })();

    // Listen for navigation events from EnvironmentSelector
    const handleNavigate = (e: Event) => {
      const detail = (e as CustomEvent<{ nav: string; sub?: string }>).detail;
      activeNav = detail.nav;
      if (detail.sub) {
        appSettingsSub = detail.sub;
      }
    };
    window.addEventListener("navigate", handleNavigate);
    return () => window.removeEventListener("navigate", handleNavigate);
  });

  // Load data when connection becomes active
  $effect(() => {
    if (connectionStore.status === "connected" && connectionStore.client) {
      configStore.loadUserConfig();
      projectsStore.loadProjects();
      pluginsStore.loadPlugins();
      skillsStore.loadSkills();
      memoryStore.loadProjects();
      mcpStore.loadServers();

      connectionStore.wsClient?.onEvent((event) => {
        if (event.type === "configChanged") {
          void configStore.loadUserConfig();
          if (projectsStore.activeProjectId) {
            void configStore.loadProjectConfig(projectsStore.activeProjectId);
          }
        }
      });
    }
  });

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  function isSettingsModule(): boolean {
    return activeNav === "S";
  }

  function isPluginsModule(): boolean {
    return activeNav === "P";
  }

  function isSkillsModule(): boolean {
    return activeNav === "K";
  }

  function isMemoryModule(): boolean {
    return activeNav === "M";
  }

  function isMcpModule(): boolean {
    return activeNav === "C";
  }

  function isEffectiveConfigModule(): boolean {
    return activeNav === "E";
  }

  function isLauncherModule(): boolean {
    return activeNav === "L";
  }

  function isAppSettingsModule(): boolean {
    return activeNav === "A";
  }
</script>

<!-- ===== Root container ===== -->
<div class="flex h-screen w-screen flex-col overflow-hidden bg-gray-950 text-gray-100">

  <!-- ===== Header ===== -->
  <header class="flex items-center justify-between border-b border-gray-800 bg-gray-900 px-4 py-2">
    <span class="text-sm font-semibold text-gray-100">dot-claude</span>

    <div class="flex items-center gap-2">
      <EnvironmentSelector />
      <span class="text-gray-600">→</span>
      <ScopeSelector />
    </div>

    <ConnectionStatus />
  </header>

  <!-- ===== Body (three-panel layout) ===== -->
  <div class="flex flex-1 overflow-hidden">

    {#if connectionStore.status === "disconnected" && connectionStore.error}
      <!-- Not connected — full-width message -->
      <div class="flex flex-1 items-center justify-center">
        <div class="text-center">
          <p class="text-sm font-medium text-gray-300">Not connected</p>
          {#if connectionStore.error}
            <p class="mt-1 text-xs text-red-400">{connectionStore.error}</p>
          {/if}
        </div>
      </div>
    {:else}

      <!-- Sidebar: icon nav -->
      <nav
        class="flex flex-col items-center border-r border-gray-800 bg-gray-900 py-3"
        style="width: var(--sidebar-width, 3.5rem)"
      >
        <!-- Main nav buttons -->
        <div class="flex flex-col items-center gap-1 flex-1">
          {#each navButtons as btn}
            <button
              class="flex h-10 w-10 items-center justify-center rounded-lg text-sm font-semibold transition-colors
                {activeNav === btn.id
                ? 'bg-blue-600 text-white'
                : 'text-gray-400 hover:bg-gray-800 hover:text-gray-100'}"
              title={btn.label}
              onclick={() => { activeNav = btn.id; }}
            >
              {btn.id}
            </button>
          {/each}
        </div>

        <!-- App Settings button (separated at bottom) -->
        <div class="mt-2 border-t border-gray-800 pt-2">
          <button
            class="flex h-10 w-10 items-center justify-center rounded-lg text-sm font-semibold transition-colors
              {activeNav === appSettingsButton.id
              ? 'bg-blue-600 text-white'
              : 'text-gray-400 hover:bg-gray-800 hover:text-gray-100'}"
            title={appSettingsButton.label}
            onclick={() => { activeNav = appSettingsButton.id; }}
          >
            {appSettingsButton.id}
          </button>
        </div>
      </nav>

      <!-- Sub-panel: list -->
      <aside
        class="flex flex-col border-r border-gray-800 bg-gray-900"
        style="width: var(--subpanel-width, 14rem)"
      >
        <div class="border-b border-gray-800 px-4 py-3">
          <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400">
            {navButtons.find((b) => b.id === activeNav)?.label ?? (activeNav === appSettingsButton.id ? appSettingsButton.label : "")}
          </h2>
        </div>

        {#if isSettingsModule()}
          <!-- Settings sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each settingsSections as section}
              <li>
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors
                    {settingsSection === section.id
                    ? 'bg-gray-800 text-white'
                    : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                  onclick={() => { settingsSection = section.id; }}
                >
                  {section.label}
                </button>
              </li>
            {/each}
          </ul>
        {:else if isPluginsModule()}
          <!-- Plugins sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each pluginsSections as section}
              <li>
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors
                    {pluginsSection === section.id
                    ? 'bg-gray-800 text-white'
                    : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                  onclick={() => { pluginsSection = section.id; }}
                >
                  {section.label}
                </button>
              </li>
            {/each}
          </ul>
        {:else if isSkillsModule()}
          <!-- Skills sub-navigation: list skill names -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#if skillsStore.loading}
              <li class="px-4 py-2 text-xs text-gray-500">Loading...</li>
            {:else if skillsStore.skills.length === 0}
              <li class="px-4 py-2 text-xs text-gray-600">No skills found</li>
            {:else}
              {#each skillsStore.skills as skill (skill.id)}
                <li>
                  <button
                    class="flex w-full items-center justify-between px-4 py-2 text-left text-sm transition-colors
                      {skillsStore.selectedSkillId === skill.id
                      ? 'bg-gray-800 text-white'
                      : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                    onclick={() => skillsStore.selectSkill(skill.id)}
                  >
                    <span class="truncate">{skill.name}</span>
                    {#if skill.valid}
                      <span class="ml-1 flex-shrink-0 text-xs text-green-400">✓</span>
                    {:else}
                      <span class="ml-1 flex-shrink-0 text-xs text-red-400">✗</span>
                    {/if}
                  </button>
                </li>
              {/each}
            {/if}
          </ul>
        {:else if isMemoryModule()}
          <!-- Memory sub-panel: project selector + file list -->
          <MemoryList />
        {:else if isMcpModule()}
          <!-- MCP sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each mcpSections as section}
              <li>
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors
                    {mcpSection === section.id
                    ? 'bg-gray-800 text-white'
                    : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                  onclick={() => { mcpSection = section.id; }}
                >
                  {section.label}
                </button>
              </li>
            {/each}
          </ul>
        {:else if isEffectiveConfigModule()}
          <!-- Effective Config: no sub-navigation needed -->
          <div class="flex-1 overflow-y-auto py-2">
            <p class="px-4 py-2 text-xs text-gray-600">Merged config for active project</p>
          </div>
        {:else if isLauncherModule()}
          <!-- Launcher: no sub-navigation needed -->
          <div class="flex-1 overflow-y-auto py-2">
            <p class="px-4 py-2 text-xs text-gray-600">Select a project and launch</p>
          </div>
        {:else if isAppSettingsModule()}
          <!-- App Settings sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            <li>
              <button
                class="w-full px-4 py-2 text-left text-sm transition-colors
                  {appSettingsSub === 'appearance'
                  ? 'bg-gray-800 text-white'
                  : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                onclick={() => { appSettingsSub = "appearance"; }}
              >
                外观
              </button>
            </li>
            <li>
              <button
                class="w-full px-4 py-2 text-left text-sm transition-colors
                  {appSettingsSub === 'connections'
                  ? 'bg-gray-800 text-white'
                  : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                onclick={() => { appSettingsSub = "connections"; }}
              >
                连接
              </button>
            </li>
          </ul>
        {/if}
      </aside>

      <!-- Detail panel -->
      <main class="flex flex-1 flex-col overflow-hidden">
        <div class="flex flex-1 flex-col overflow-hidden">
          {#if isAppSettingsModule()}
            <!-- App Settings module: always accessible, regardless of connection -->
            <AppSettingsView activeSub={appSettingsSub} />

          {:else if connectionStore.status === "connecting"}
            <div class="flex-1 overflow-auto p-6">
              <p class="text-sm text-gray-500">Connecting to daemon...</p>
            </div>

          {:else if connectionStore.status === "disconnected"}
            <div class="flex flex-1 items-center justify-center">
              <p class="text-sm text-gray-500">Not connected</p>
            </div>

          {:else if isSettingsModule()}
            <!-- Settings module: SettingsEditor orchestrator -->
            {#if configStore.loading}
              <div class="flex-1 overflow-auto p-6">
                <p class="text-sm text-gray-500">Loading config...</p>
              </div>
            {:else}
              <SettingsEditor activeSection={settingsSection} />
            {/if}

          {:else if isPluginsModule()}
            <!-- Plugins module: PluginsModule orchestrator -->
            <PluginsModule activeSection={pluginsSection} />

          {:else if isSkillsModule()}
            <!-- Skills module: SkillsModule orchestrator -->
            <SkillsModule />

          {:else if isMemoryModule()}
            <!-- Memory module: MemoryModule orchestrator -->
            <MemoryModule />

          {:else if isMcpModule()}
            <!-- MCP module: McpModule orchestrator -->
            <McpModule activeSection={mcpSection} />

          {:else if isEffectiveConfigModule()}
            <!-- Effective Config module -->
            <EffectiveConfigView />

          {:else if isLauncherModule()}
            <!-- Launcher module -->
            <LauncherView />

          {:else}
            <div class="flex flex-1 items-center justify-center">
              <p class="text-sm text-gray-600">Select an item to view details</p>
            </div>
          {/if}
        </div>
      </main>

    {/if}
  </div>
</div>

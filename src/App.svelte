<script lang="ts">
  import { onMount } from "svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import ConnectionStatus from "$lib/components/shared/ConnectionStatus.svelte";
  import SettingsEditor from "$lib/components/settings/SettingsEditor.svelte";
  import PluginsModule from "$lib/components/plugins/PluginsModule.svelte";

  // ---------------------------------------------------------------------------
  // Constants (dev defaults — swap for real config/env later)
  // ---------------------------------------------------------------------------

  const DAEMON_BASE_URL = "http://localhost:7890";
  const DAEMON_TOKEN = "dev-token";

  // ---------------------------------------------------------------------------
  // Navigation state
  // ---------------------------------------------------------------------------

  let activeNav = $state("S");
  let activeItem = $state(0);

  const navButtons = [
    { id: "S", label: "Sessions" },
    { id: "P", label: "Projects" },
    { id: "K", label: "Hooks" },
    { id: "M", label: "MCP" },
    { id: "C", label: "Config" },
    { id: "G", label: "Plugins" },
    { id: "E", label: "Environment" },
    { id: "L", label: "Logs" },
    { id: "A", label: "About" },
  ];

  const subItems: Record<string, string[]> = {
    S: ["Session 1", "Session 2", "Session 3"],
    P: ["Project Alpha", "Project Beta"],
    K: ["pre-tool", "post-tool", "pre-compact"],
    M: ["filesystem", "github", "postgres"],
    E: ["Variables", "Secrets"],
    L: ["daemon.log", "tauri.log"],
    A: ["Version", "License"],
  };

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
  // Derived: active project options for header dropdown
  // ---------------------------------------------------------------------------

  let selectedProjectId = $state<string>("");

  // ---------------------------------------------------------------------------
  // Mount: connect + load initial data
  // ---------------------------------------------------------------------------

  onMount(() => {
    void (async () => {
      await connectionStore.connect(DAEMON_BASE_URL, DAEMON_TOKEN);

      if (connectionStore.status === "connected") {
        await Promise.all([
          configStore.loadUserConfig(),
          projectsStore.loadProjects(),
          pluginsStore.loadPlugins(),
        ]);
      }

      // Subscribe to WS events for config_changed → reload config
      if (connectionStore.wsClient) {
        connectionStore.wsClient.onEvent((event) => {
          if (event.type === "configChanged") {
            void configStore.loadUserConfig();
          }
        });
      }
    })();
  });

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  function isSettingsModule(): boolean {
    return activeNav === "C";
  }

  function isPluginsModule(): boolean {
    return activeNav === "G";
  }
</script>

<!-- ===== Root container ===== -->
<div class="flex h-screen w-screen flex-col overflow-hidden bg-gray-950 text-gray-100">

  <!-- ===== Header ===== -->
  <header class="flex items-center justify-between border-b border-gray-800 bg-gray-900 px-4 py-2">
    <div class="flex items-center gap-3">
      <span class="text-sm font-semibold text-gray-100">dot-claude</span>

      <!-- Project selector -->
      {#if projectsStore.projects.length > 0}
        <select
          class="rounded bg-gray-800 px-2 py-1 text-xs text-gray-300 focus:outline-none"
          bind:value={selectedProjectId}
          onchange={() => projectsStore.selectProject(selectedProjectId || null)}
        >
          <option value="">No project</option>
          {#each projectsStore.projects as project}
            <option value={project.id}>{project.name}</option>
          {/each}
        </select>
      {:else}
        <span class="text-xs text-gray-600">No projects</span>
      {/if}
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
        class="flex flex-col items-center gap-1 border-r border-gray-800 bg-gray-900 py-3"
        style="width: var(--sidebar-width, 3.5rem)"
      >
        {#each navButtons as btn}
          <button
            class="flex h-10 w-10 items-center justify-center rounded-lg text-sm font-semibold transition-colors
              {activeNav === btn.id
              ? 'bg-blue-600 text-white'
              : 'text-gray-400 hover:bg-gray-800 hover:text-gray-100'}"
            title={btn.label}
            onclick={() => { activeNav = btn.id; activeItem = 0; }}
          >
            {btn.id}
          </button>
        {/each}
      </nav>

      <!-- Sub-panel: list -->
      <aside
        class="flex flex-col border-r border-gray-800 bg-gray-900"
        style="width: var(--subpanel-width, 14rem)"
      >
        <div class="border-b border-gray-800 px-4 py-3">
          <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400">
            {navButtons.find((b) => b.id === activeNav)?.label ?? ""}
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
        {:else}
          <!-- Generic sub-item list -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each (subItems[activeNav] ?? []) as item, i}
              <li>
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors
                    {activeItem === i
                    ? 'bg-gray-800 text-white'
                    : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
                  onclick={() => { activeItem = i; }}
                >
                  {item}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </aside>

      <!-- Detail panel -->
      <main class="flex flex-1 flex-col overflow-hidden">
        {#if !isSettingsModule() && !isPluginsModule()}
          <div class="border-b border-gray-800 px-6 py-3">
            <h1 class="text-sm font-medium text-gray-200">
              {subItems[activeNav]?.[activeItem] ?? "—"}
            </h1>
          </div>
        {/if}

        <div class="flex flex-1 flex-col overflow-hidden">
          {#if connectionStore.status === "connecting"}
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

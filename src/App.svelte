<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { memoryStore } from "$lib/stores/memory.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { onConfigChanged } from "$lib/ipc/events.js";
  import ScopeSelector from "$lib/components/shared/ScopeSelector.svelte";
import ResizeHandle from "$lib/components/shared/ResizeHandle.svelte";
  import SettingsEditor from "$lib/components/settings/SettingsEditor.svelte";
  import PluginsModule from "$lib/components/plugins/PluginsModule.svelte";
  import SkillsModule from "$lib/components/skills/SkillsModule.svelte";
  import SkillList from "$lib/components/skills/SkillList.svelte";
  import MemoryList from "$lib/components/memory/MemoryList.svelte";
  import MemoryModule from "$lib/components/memory/MemoryModule.svelte";
  import McpModule from "$lib/components/mcp/McpModule.svelte";
  import EffectiveConfigView from "$lib/components/effective/EffectiveConfigView.svelte";
  import LauncherView from "$lib/components/launcher/LauncherView.svelte";
  import AppSettingsView from "$lib/components/appsettings/AppSettingsView.svelte";
  import ClaudeMdList from "$lib/components/claudemd/ClaudeMdList.svelte";
  import ClaudeMdModule from "$lib/components/claudemd/ClaudeMdModule.svelte";
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";
  import Toast from "$lib/components/shared/Toast.svelte";

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

  // Font size effect
  $effect(() => {
    document.documentElement.style.setProperty("--app-font-size", appSettingsStore.preferences.fontSize + "px");
  });

  // Panel width effects
  $effect(() => {
    document.documentElement.style.setProperty("--sidebar-width", appSettingsStore.preferences.sidebarWidth + "px");
  });
  $effect(() => {
    document.documentElement.style.setProperty("--subpanel-width", appSettingsStore.preferences.subpanelWidth + "px");
  });

  // ---------------------------------------------------------------------------
  // Navigation state
  // ---------------------------------------------------------------------------

  let activeNav = $state("S");

  const navButtons = [
    { id: "S", label: "设置", icon: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" },
    { id: "P", label: "插件", icon: "M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875s-2.25.84-2.25 1.875c0 .369.128.713.349 1.003.215.283.401.604.401.959v0a.64.64 0 0 1-.657.643 48.39 48.39 0 0 1-4.163-.3c.186 1.613.293 3.25.315 4.907a.656.656 0 0 1-.658.663v0c-.355 0-.676-.186-.959-.401a1.647 1.647 0 0 0-1.003-.349c-1.036 0-1.875 1.007-1.875 2.25s.84 2.25 1.875 2.25c.369 0 .713-.128 1.003-.349.283-.215.604-.401.959-.401v0c.31 0 .555.26.532.57a48.039 48.039 0 0 1-.642 5.056c1.518.19 3.058.309 4.616.354a.64.64 0 0 0 .657-.643v0c0-.355-.186-.676-.401-.959a1.647 1.647 0 0 1-.349-1.003c0-1.035 1.008-1.875 2.25-1.875 1.243 0 2.25.84 2.25 1.875 0 .369-.128.713-.349 1.003-.215.283-.4.604-.4.959v0c0 .333.277.599.61.58a48.1 48.1 0 0 0 5.427-.63 48.05 48.05 0 0 0 .582-4.717.532.532 0 0 0-.533-.57v0c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.035 0-1.875-1.007-1.875-2.25s.84-2.25 1.875-2.25c.37 0 .713.128 1.003.349.283.215.604.401.96.401v0a.656.656 0 0 0 .657-.663 47.703 47.703 0 0 0-.31-4.82 47.872 47.872 0 0 1-4.164.3.64.64 0 0 1-.657-.643v0Z" },
    { id: "K", label: "技能", icon: "m3.75 13.5 10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75Z" },
    { id: "M", label: "记忆", icon: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" },
    { id: "D", label: "指令", icon: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" },
    { id: "C", label: "MCP", icon: "M5.25 14.25h13.5m-13.5 0a3 3 0 0 1-3-3m3 3a3 3 0 1 0 0 6h13.5a3 3 0 1 0 0-6m-16.5-3a3 3 0 0 1 3-3h13.5a3 3 0 0 1 3 3m-19.5 0a4.5 4.5 0 0 1 .9-2.7L5.737 5.1a3.375 3.375 0 0 1 2.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 0 1 .9 2.7m0 0h.375a2.625 2.625 0 0 1 0 5.25H17.25m-13.5-5.25H3.375a2.625 2.625 0 0 0 0 5.25H6.75m0-5.25v5.25m11.25-5.25v5.25" },
    { id: "E", label: "配置", icon: "M10.5 6h9.75M10.5 6a1.5 1.5 0 1 1-3 0m3 0a1.5 1.5 0 1 0-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-9.75 0h9.75" },
    { id: "L", label: "启动", icon: "M15.59 14.37a6 6 0 0 1-5.84 7.38v-4.8m5.84-2.58a14.98 14.98 0 0 0 6.16-12.12A14.98 14.98 0 0 0 9.631 8.41m5.96 5.96a14.926 14.926 0 0 1-5.841 2.58m-.119-8.54a6 6 0 0 0-7.381 5.84h4.8m2.581-5.84a14.927 14.927 0 0 0-2.58 5.84m2.699 2.7c-.103.021-.207.041-.311.06a15.09 15.09 0 0 1-2.448-2.448 14.9 14.9 0 0 1 .06-.312m-2.24 2.39a4.493 4.493 0 0 0-1.757 4.306 4.493 4.493 0 0 0 4.306-1.758M16.5 9a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Z" },
  ];

  // App Settings is kept separate (bottom of sidebar)
  const appSettingsButton = { id: "A", label: "设置", icon: "M11.42 15.17 17.25 21A2.652 2.652 0 0 0 21 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 1 1-3.586-3.586l5.653-4.655m5.976-.833a3 3 0 0 1-4.243-4.243M11.42 15.17l1.434-1.74" };

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
  // Mount: load all stores and subscribe to file-watcher events
  // ---------------------------------------------------------------------------

  // Store cleanup functions for async subscriptions
  let unlistenConfigChanged: (() => void) | undefined;

  onMount(() => {
    void (async () => {
      // Load UI preferences first
      await appSettingsStore.load();

      // IPC is always available — load all data stores in parallel
      await Promise.all([
        configStore.loadUserConfig(),
        projectsStore.loadProjects(),
        pluginsStore.loadPlugins(),
        skillsStore.loadSkills(),
        memoryStore.loadProjects(),
        mcpStore.loadServers(),
        claudeMdStore.loadFiles(),
      ]);

      // Subscribe to config-changed events pushed from backend file watcher
      unlistenConfigChanged = await onConfigChanged((payload) => {
        // Update user config cache on any config change (file-watcher or API save)
        configStore.setUserConfig(payload.settings);
      });
    })();

    // Listen for navigation events
    const handleNavigate = (e: Event) => {
      const detail = (e as CustomEvent<{ nav: string; sub?: string }>).detail;
      activeNav = detail.nav;
      if (detail.sub) {
        appSettingsSub = detail.sub;
      }
    };
    window.addEventListener("navigate", handleNavigate);

    return () => {
      unlistenConfigChanged?.();
      window.removeEventListener("navigate", handleNavigate);
    };
  });

  // ---------------------------------------------------------------------------
  // Sidebar collapse
  // ---------------------------------------------------------------------------

  const SIDEBAR_COLLAPSED = 56;
  const SIDEBAR_EXPANDED = 140;
  let sidebarCollapsed = $derived(appSettingsStore.preferences.sidebarWidth <= SIDEBAR_COLLAPSED);

  function toggleSidebar() {
    const next = sidebarCollapsed ? SIDEBAR_EXPANDED : SIDEBAR_COLLAPSED;
    appSettingsStore.update({ sidebarWidth: next });
  }

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  // Navigation checks are done directly in the template as `activeNav === "X"`
  // to avoid any Svelte 5 reactivity tracking issues with function calls or $derived.
</script>

<!-- ===== Root container ===== -->
<div class="flex h-screen w-screen flex-col overflow-hidden" style="background-color: var(--bg-primary); color: var(--text-primary)">

  <!-- ===== Header ===== -->
  <header class="flex items-center justify-between px-4 py-2" style="background-color: var(--bg-secondary); border-bottom: 1px solid var(--border-color)">
    <span class="text-sm font-semibold" style="color: var(--text-primary)">dot-claude</span>

    <div class="flex items-center gap-2">
      <ScopeSelector />
    </div>
  </header>

  <!-- ===== Body (three-panel layout) ===== -->
  <div class="flex flex-1 overflow-hidden">

    <!-- Sidebar: icon + text nav -->
      <nav
        class="flex flex-col overflow-hidden"
        style="width: var(--sidebar-width); flex-shrink: 0; background-color: var(--bg-secondary); border-right: 1px solid var(--border-color); transition: width 0.15s ease"
      >
        <!-- Nav items (including App Settings) -->
        <div class="flex flex-col gap-1 flex-1 px-2 py-3 overflow-y-auto">
          {#each [...navButtons, appSettingsButton] as btn}
            <button
              class="flex items-center gap-2 h-9 px-2 rounded-lg text-sm transition-colors overflow-hidden whitespace-nowrap
                {activeNav === btn.id
                ? ''
                : 'hover:text-[var(--text-primary)]'}"
              style="{activeNav === btn.id ? 'background-color: var(--nav-active-bg)' : ''} color: {activeNav === btn.id ? 'var(--nav-active-text)' : 'var(--text-secondary)'}"
              title={btn.label}
              onclick={() => { activeNav = btn.id; }}
            >
              <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" d={btn.icon} />
              </svg>
              <span class="truncate">{btn.label}</span>
            </button>
          {/each}
        </div>

        <!-- Collapse / expand toggle — alone at bottom -->
        <button
          class="flex items-center h-8 px-2 mx-2 mb-2 rounded-lg transition-colors hover:bg-[var(--bg-card-hover)]"
          style="color: var(--text-muted); flex-shrink: 0"
          title={sidebarCollapsed ? "展开侧边栏" : "折叠侧边栏"}
          onclick={toggleSidebar}
        >
          <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            {#if sidebarCollapsed}
              <!-- Panel expand: right-facing sidebar icon -->
              <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 3.75h16.5v16.5H3.75z" />
              <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 3.75v16.5" />
            {:else}
              <!-- Panel collapse: left-facing sidebar icon -->
              <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 3.75h16.5v16.5H3.75z" />
              <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 3.75v16.5" />
            {/if}
          </svg>
        </button>
      </nav>

      <!-- Sidebar resize handle -->
      <ResizeHandle
        min={56}
        max={180}
        onResize={(w) => appSettingsStore.update({ sidebarWidth: w })}
      />

      <!-- Sub-panel: list -->
      <aside
        class="flex flex-col"
        style="width: var(--subpanel-width); flex-shrink: 0; background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)"
      >
        {#if activeNav !== "K"}
          <div class="px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
            <h2 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
              {navButtons.find((b) => b.id === activeNav)?.label ?? (activeNav === appSettingsButton.id ? appSettingsButton.label : "")}
            </h2>
          </div>
        {/if}

        {#if activeNav === "S"}
          <!-- Settings sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each settingsSections as section}
              <li>
                {#if settingsSection === section.id}
                  <button
                    class="w-full px-4 py-2 text-left text-sm transition-colors"
                    style="background-color: var(--accent-bg); color: var(--text-primary)"
                    onclick={() => { settingsSection = section.id; }}
                  >
                    {section.label}
                  </button>
                {:else}
                  <button
                    class="w-full px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
                    style="color: var(--text-secondary)"
                    onclick={() => { settingsSection = section.id; }}
                  >
                    {section.label}
                  </button>
                {/if}
              </li>
            {/each}
          </ul>
        {:else if activeNav === "P"}
          <!-- Plugins sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each pluginsSections as section}
              <li>
                {#if pluginsSection === section.id}
                  <button
                    class="w-full px-4 py-2 text-left text-sm transition-colors"
                    style="background-color: var(--accent-bg); color: var(--text-primary)"
                    onclick={() => { pluginsSection = section.id; }}
                  >
                    {section.label}
                  </button>
                {:else}
                  <button
                    class="w-full px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
                    style="color: var(--text-secondary)"
                    onclick={() => { pluginsSection = section.id; }}
                  >
                    {section.label}
                  </button>
                {/if}
              </li>
            {/each}
          </ul>
        {:else if activeNav === "K"}
          <!-- Skills sub-panel -->
          <SkillList />
        {:else if activeNav === "M"}
          <!-- Memory sub-panel: project selector + file list -->
          <MemoryList />
        {:else if activeNav === "D"}
          <ClaudeMdList />
        {:else if activeNav === "C"}
          <!-- MCP sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            {#each mcpSections as section}
              <li>
                {#if mcpSection === section.id}
                  <button
                    class="w-full px-4 py-2 text-left text-sm transition-colors"
                    style="background-color: var(--accent-bg); color: var(--text-primary)"
                    onclick={() => { mcpSection = section.id; }}
                  >
                    {section.label}
                  </button>
                {:else}
                  <button
                    class="w-full px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
                    style="color: var(--text-secondary)"
                    onclick={() => { mcpSection = section.id; }}
                  >
                    {section.label}
                  </button>
                {/if}
              </li>
            {/each}
          </ul>
        {:else if activeNav === "E"}
          <!-- Effective Config: no sub-navigation needed -->
          <div class="flex-1 overflow-y-auto py-2">
            <p class="px-4 py-2 text-xs" style="color: var(--text-muted)">Merged config for active project</p>
          </div>
        {:else if activeNav === "L"}
          <!-- Launcher: no sub-navigation needed -->
          <div class="flex-1 overflow-y-auto py-2">
            <p class="px-4 py-2 text-xs" style="color: var(--text-muted)">Select a project and launch</p>
          </div>
        {:else if activeNav === "A"}
          <!-- App Settings sub-navigation -->
          <ul class="flex-1 overflow-y-auto py-2">
            <li>
              {#if appSettingsSub === "appearance"}
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors"
                  style="background-color: var(--accent-bg); color: var(--text-primary)"
                  onclick={() => { appSettingsSub = "appearance"; }}
                >
                  外观
                </button>
              {:else}
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
                  style="color: var(--text-secondary)"
                  onclick={() => { appSettingsSub = "appearance"; }}
                >
                  外观
                </button>
              {/if}
            </li>
            <li>
              {#if appSettingsSub === "connections"}
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors"
                  style="background-color: var(--accent-bg); color: var(--text-primary)"
                  onclick={() => { appSettingsSub = "connections"; }}
                >
                  连接
                </button>
              {:else}
                <button
                  class="w-full px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
                  style="color: var(--text-secondary)"
                  onclick={() => { appSettingsSub = "connections"; }}
                >
                  连接
                </button>
              {/if}
            </li>
          </ul>
        {/if}
      </aside>

      <!-- Sub-panel resize handle -->
      <ResizeHandle
        min={160}
        max={400}
        onResize={(w) => appSettingsStore.update({ subpanelWidth: w })}
      />

      <!-- Detail panel -->
      <main class="flex flex-1 flex-col overflow-hidden">
        <div class="flex flex-1 flex-col overflow-hidden">
          {#if activeNav === "A"}
            <!-- App Settings module -->
            <AppSettingsView />

          {:else if activeNav === "S"}
            <!-- Settings module: SettingsEditor orchestrator -->
            {#if configStore.loading}
              <div class="flex-1 overflow-auto p-6">
                <p class="text-sm" style="color: var(--text-muted)">Loading config...</p>
              </div>
            {:else}
              <SettingsEditor activeSection={settingsSection} />
            {/if}

          {:else if activeNav === "P"}
            <!-- Plugins module: PluginsModule orchestrator -->
            <PluginsModule activeSection={pluginsSection} />

          {:else if activeNav === "K"}
            <!-- Skills module: SkillsModule orchestrator -->
            <SkillsModule />

          {:else if activeNav === "M"}
            <!-- Memory module: MemoryModule orchestrator -->
            <MemoryModule />

          {:else if activeNav === "D"}
            <ClaudeMdModule />

          {:else if activeNav === "C"}
            <!-- MCP module: McpModule orchestrator -->
            <McpModule activeSection={mcpSection} />

          {:else if activeNav === "E"}
            <!-- Effective Config module -->
            <EffectiveConfigView />

          {:else if activeNav === "L"}
            <!-- Launcher module -->
            <LauncherView />

          {:else}
            <div class="flex flex-1 items-center justify-center">
              <p class="text-sm" style="color: var(--text-muted)">Select an item to view details</p>
            </div>
          {/if}
        </div>
      </main>

  </div>
  <Toast />
</div>

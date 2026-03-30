<script lang="ts">
  // Svelte 5 runes
  let activeNav = $state("S");
  let activeItem = $state(0);

  const navButtons = [
    { id: "S", label: "Sessions" },
    { id: "P", label: "Projects" },
    { id: "K", label: "Hooks" },
    { id: "M", label: "MCP" },
    { id: "C", label: "Config" },
    { id: "E", label: "Environment" },
    { id: "L", label: "Logs" },
    { id: "A", label: "About" },
  ];

  const subItems: Record<string, string[]> = {
    S: ["Session 1", "Session 2", "Session 3"],
    P: ["Project Alpha", "Project Beta"],
    K: ["pre-tool", "post-tool", "pre-compact"],
    M: ["filesystem", "github", "postgres"],
    C: ["Global Config", "Project Config"],
    E: ["Variables", "Secrets"],
    L: ["daemon.log", "tauri.log"],
    A: ["Version", "License"],
  };
</script>

<div class="flex h-screen w-screen overflow-hidden bg-gray-950 text-gray-100">
  <!-- Sidebar: icon nav -->
  <nav
    class="flex flex-col items-center gap-1 border-r border-gray-800 bg-gray-900 py-3"
    style="width: var(--sidebar-width)"
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
    style="width: var(--subpanel-width)"
  >
    <div class="border-b border-gray-800 px-4 py-3">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400">
        {navButtons.find((b) => b.id === activeNav)?.label ?? ""}
      </h2>
    </div>
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
  </aside>

  <!-- Detail panel -->
  <main class="flex flex-1 flex-col overflow-hidden">
    <div class="border-b border-gray-800 px-6 py-3">
      <h1 class="text-sm font-medium text-gray-200">
        {subItems[activeNav]?.[activeItem] ?? "—"}
      </h1>
    </div>
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">
        Select an item to view details
      </p>
    </div>
  </main>
</div>

<script lang="ts">
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { DaemonClient } from "$lib/api/client";

  // -------------------------------------------------------------------------
  // Local UI state
  // -------------------------------------------------------------------------

  let showToken = $state(false);
  let testStatus = $state<"idle" | "testing" | "ok" | "error">("idle");
  let testMessage = $state("");

  let newProjectPath = $state("");
  let addProjectError = $state("");
  let addingProject = $state(false);

  // -------------------------------------------------------------------------
  // Helpers
  // -------------------------------------------------------------------------

  async function testConnection() {
    testStatus = "testing";
    testMessage = "";
    try {
      const client = new DaemonClient(
        appSettingsStore.preferences.daemonUrl,
        appSettingsStore.preferences.daemonToken,
      );
      const health = await client.health();
      testStatus = "ok";
      testMessage = `Connected — daemon v${health.version}`;
    } catch (e) {
      testStatus = "error";
      testMessage = e instanceof Error ? e.message : "Connection failed";
    }
  }

  async function addProject() {
    const path = newProjectPath.trim();
    if (!path) return;
    addingProject = true;
    addProjectError = "";
    try {
      await projectsStore.registerProject(path);
      newProjectPath = "";
    } catch (e) {
      addProjectError = e instanceof Error ? e.message : "Failed to add project";
    } finally {
      addingProject = false;
    }
  }

  async function removeProject(id: string) {
    try {
      await projectsStore.unregisterProject(id);
    } catch { /* ignore */ }
  }
</script>

<div class="flex flex-1 flex-col overflow-y-auto p-6 space-y-8">

  <!-- ===== Appearance ===== -->
  <section class="space-y-4">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400">Appearance</h2>

    <!-- Theme -->
    <div class="space-y-1.5">
      <label for="theme-select" class="block text-xs font-medium text-gray-400">Theme</label>
      <select
        id="theme-select"
        class="rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-200 focus:border-blue-500 focus:outline-none"
        value={appSettingsStore.preferences.theme}
        onchange={(e) => appSettingsStore.update({ theme: (e.currentTarget as HTMLSelectElement).value as "light" | "dark" | "system" })}
      >
        <option value="light">Light</option>
        <option value="dark">Dark</option>
        <option value="system">System</option>
      </select>
    </div>

    <!-- Font size -->
    <div class="space-y-1.5">
      <label for="font-size" class="block text-xs font-medium text-gray-400">
        Font Size: {appSettingsStore.preferences.fontSize}px
      </label>
      <input
        id="font-size"
        type="range"
        min="12"
        max="20"
        step="1"
        value={appSettingsStore.preferences.fontSize}
        oninput={(e) => appSettingsStore.update({ fontSize: Number((e.currentTarget as HTMLInputElement).value) })}
        class="w-48 accent-blue-500"
      />
      <div class="flex w-48 justify-between text-xs text-gray-600">
        <span>12</span>
        <span>20</span>
      </div>
    </div>
  </section>

  <!-- ===== Connection ===== -->
  <section class="space-y-4">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400">Connection</h2>

    <!-- Daemon URL -->
    <div class="space-y-1.5">
      <label for="daemon-url" class="block text-xs font-medium text-gray-400">Daemon URL</label>
      <input
        id="daemon-url"
        type="text"
        class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-200 placeholder-gray-600 focus:border-blue-500 focus:outline-none"
        value={appSettingsStore.preferences.daemonUrl}
        oninput={(e) => appSettingsStore.update({ daemonUrl: (e.currentTarget as HTMLInputElement).value })}
        placeholder="http://127.0.0.1:7890"
      />
    </div>

    <!-- Daemon Token -->
    <div class="space-y-1.5">
      <label for="daemon-token" class="block text-xs font-medium text-gray-400">Daemon Token</label>
      <div class="flex gap-2">
        <input
          id="daemon-token"
          type={showToken ? "text" : "password"}
          class="flex-1 rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-200 placeholder-gray-600 focus:border-blue-500 focus:outline-none"
          value={appSettingsStore.preferences.daemonToken}
          oninput={(e) => appSettingsStore.update({ daemonToken: (e.currentTarget as HTMLInputElement).value })}
          placeholder="dev-token"
        />
        <button
          class="flex-shrink-0 rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-xs text-gray-400 hover:bg-gray-700 hover:text-gray-200 transition-colors"
          onclick={() => { showToken = !showToken; }}
          title={showToken ? "Hide token" : "Show token"}
        >
          {showToken ? "Hide" : "Show"}
        </button>
      </div>
    </div>

    <!-- Test Connection -->
    <div class="flex items-center gap-3">
      <button
        class="rounded-lg bg-gray-700 px-4 py-2 text-sm text-gray-200 hover:bg-gray-600 transition-colors disabled:opacity-50"
        onclick={testConnection}
        disabled={testStatus === "testing"}
      >
        {testStatus === "testing" ? "Testing..." : "Test Connection"}
      </button>

      {#if testStatus === "ok"}
        <span class="text-xs text-green-400">{testMessage}</span>
      {:else if testStatus === "error"}
        <span class="text-xs text-red-400">{testMessage}</span>
      {/if}
    </div>
  </section>

  <!-- ===== Projects ===== -->
  <section class="space-y-4">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-400">Projects</h2>

    <!-- Project list -->
    {#if projectsStore.projects.length === 0}
      <p class="text-xs text-gray-600 italic">No projects registered.</p>
    {:else}
      <ul class="space-y-2">
        {#each projectsStore.projects as project (project.id)}
          <li class="flex items-center justify-between rounded-lg border border-gray-700 bg-gray-900 px-4 py-3">
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium text-gray-200">{project.name}</p>
              <p class="truncate text-xs text-gray-500">{project.path}</p>
            </div>
            <button
              class="ml-4 flex-shrink-0 rounded px-2 py-1 text-xs text-red-400 hover:bg-red-900/30 hover:text-red-300 transition-colors"
              onclick={() => removeProject(project.id)}
            >
              Remove
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    <!-- Add project -->
    <div class="space-y-1.5">
      <label for="add-project-path" class="block text-xs font-medium text-gray-400">Add Project</label>
      <div class="flex gap-2">
        <input
          id="add-project-path"
          type="text"
          placeholder="/path/to/project"
          class="flex-1 rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-200 placeholder-gray-600 focus:border-blue-500 focus:outline-none font-mono"
          bind:value={newProjectPath}
          onkeydown={(e) => { if (e.key === "Enter") void addProject(); }}
        />
        <button
          class="flex-shrink-0 rounded-lg bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-500 transition-colors disabled:opacity-50"
          onclick={() => void addProject()}
          disabled={!newProjectPath.trim() || addingProject}
        >
          {addingProject ? "Adding..." : "Add"}
        </button>
      </div>
      {#if addProjectError}
        <p class="text-xs text-red-400">{addProjectError}</p>
      {/if}
    </div>
  </section>

</div>

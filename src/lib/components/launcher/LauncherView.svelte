<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";

  let selectedProjectId = $state<string>("");
  let customEnv = $state<{ key: string; value: string; enabled: boolean }[]>([]);
  let newKey = $state("");
  let newValue = $state("");
  let launching = $state(false);
  let launchResult = $state<string>("");
  let launchIsError = $state(false);

  const selectedProject = $derived(
    projectsStore.projects.find((p) => p.id === selectedProjectId),
  );

  // Build env vars from settings
  const settingsEnv = $derived(configStore.userSettings.env ?? {});

  // Active plugin count from pluginsStore
  const activePluginCount = $derived(
    pluginsStore.plugins.filter((p) => p.enabled).length,
  );

  let envChecks = $state<Record<string, boolean>>({});

  $effect(() => {
    const env = configStore.userSettings.env ?? {};
    for (const key of Object.keys(env)) {
      if (!(key in envChecks)) envChecks[key] = true;
    }
  });

  function addCustomVar() {
    const key = newKey.trim();
    if (!key) return;
    customEnv = [...customEnv, { key, value: newValue, enabled: true }];
    newKey = "";
    newValue = "";
  }

  function removeCustomVar(index: number) {
    customEnv = customEnv.filter((_, i) => i !== index);
  }

  async function launch() {
    if (!selectedProject) return;
    launching = true;
    launchResult = "";
    launchIsError = false;
    try {
      const env: Record<string, string> = {};
      // Add checked settings env vars
      for (const [k, v] of Object.entries(settingsEnv)) {
        if (envChecks[k]) env[k] = String(v);
      }
      // Add enabled custom vars
      for (const cv of customEnv) {
        if (cv.enabled && cv.key) env[cv.key] = cv.value;
      }
      await connectionStore.client!.launchClaude({
        projectPath: selectedProject.path,
        env,
      });
      launchResult = "Claude Code launched successfully!";
      launchIsError = false;
    } catch (e) {
      launchResult = `Error: ${e instanceof Error ? e.message : "Launch failed"}`;
      launchIsError = true;
    } finally {
      launching = false;
    }
  }
</script>

<div class="flex flex-1 flex-col overflow-y-auto p-6 space-y-5">

  <!-- Section heading -->
  <div>
    <h2 class="text-sm font-semibold text-gray-200">Launch Claude Code</h2>
    <p class="mt-1 text-xs text-gray-500">
      Select a project, configure environment variables, and launch.
    </p>
  </div>

  <!-- Project selector -->
  <div class="space-y-1.5">
    <label for="launcher-project" class="block text-xs font-medium text-gray-400">
      Project
    </label>
    {#if projectsStore.projects.length === 0}
      <p class="text-xs text-gray-600">No projects registered. Add one via the Projects section.</p>
    {:else}
      <select
        id="launcher-project"
        class="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-gray-200 focus:border-blue-500 focus:outline-none"
        bind:value={selectedProjectId}
      >
        <option value="">— Select a project —</option>
        {#each projectsStore.projects as project (project.id)}
          <option value={project.id}>{project.name}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- Config summary card -->
  {#if selectedProject}
    {@const settings = configStore.userSettings}
    <div class="rounded-lg border border-gray-700 bg-gray-900 p-4 space-y-2">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-400">Config Summary</h3>
      <div class="grid grid-cols-3 gap-3">
        <div class="rounded bg-gray-800 px-3 py-2 text-center">
          <p class="text-xs text-gray-500">Language</p>
          <p class="mt-0.5 text-sm font-medium text-gray-200">
            {settings.language ?? "default"}
          </p>
        </div>
        <div class="rounded bg-gray-800 px-3 py-2 text-center">
          <p class="text-xs text-gray-500">Default Mode</p>
          <p class="mt-0.5 text-sm font-medium text-gray-200">
            {settings.permissions?.defaultMode ?? "ask"}
          </p>
        </div>
        <div class="rounded bg-gray-800 px-3 py-2 text-center">
          <p class="text-xs text-gray-500">Active Plugins</p>
          <p class="mt-0.5 text-sm font-medium text-gray-200">{activePluginCount}</p>
        </div>
      </div>
      <p class="text-xs text-gray-600">Path: {selectedProject.path}</p>
    </div>
  {/if}

  <!-- Environment variables -->
  <div class="space-y-3">
    <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-400">
      Environment Variables
    </h3>

    <!-- Settings env vars as checkboxes -->
    {#if Object.keys(settingsEnv).length > 0}
      <div class="rounded-lg border border-gray-700 bg-gray-900 p-3 space-y-2">
        <p class="text-xs text-gray-500 mb-2">From user settings (check to include):</p>
        {#each Object.entries(settingsEnv) as [key, value] (key)}
          <label class="flex items-center gap-3 cursor-pointer group">
            <input
              type="checkbox"
              class="h-3.5 w-3.5 rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500 focus:ring-offset-gray-900"
              bind:checked={envChecks[key]}
            />
            <span class="flex-1 min-w-0">
              <span class="text-sm font-mono text-gray-300">{key}</span>
              <span class="mx-1 text-gray-600">=</span>
              <span class="text-sm font-mono text-gray-500 truncate">{value}</span>
            </span>
          </label>
        {/each}
      </div>
    {:else}
      <p class="text-xs text-gray-600 italic">No env vars in user settings.</p>
    {/if}

    <!-- Custom env vars -->
    {#if customEnv.length > 0}
      <div class="rounded-lg border border-gray-700 bg-gray-900 p-3 space-y-2">
        <p class="text-xs text-gray-500 mb-2">Custom variables:</p>
        {#each customEnv as cv, i (i)}
          <div class="flex items-center gap-2">
            <input
              type="checkbox"
              class="h-3.5 w-3.5 flex-shrink-0 rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500"
              bind:checked={cv.enabled}
            />
            <span class="font-mono text-sm text-gray-300">{cv.key}</span>
            <span class="text-gray-600">=</span>
            <span class="flex-1 truncate font-mono text-sm text-gray-500">{cv.value}</span>
            <button
              class="flex-shrink-0 rounded px-2 py-0.5 text-xs text-red-400 hover:bg-red-900/30 hover:text-red-300 transition-colors"
              onclick={() => removeCustomVar(i)}
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Add custom var inputs -->
    <div class="flex items-end gap-2">
      <div class="flex-1 space-y-1">
        <label for="env-key" class="block text-xs text-gray-500">Key</label>
        <input
          id="env-key"
          type="text"
          placeholder="MY_VAR"
          class="w-full rounded border border-gray-700 bg-gray-800 px-2 py-1.5 text-sm text-gray-200 placeholder-gray-600 focus:border-blue-500 focus:outline-none font-mono"
          bind:value={newKey}
          onkeydown={(e) => { if (e.key === "Enter") addCustomVar(); }}
        />
      </div>
      <div class="flex-1 space-y-1">
        <label for="env-value" class="block text-xs text-gray-500">Value</label>
        <input
          id="env-value"
          type="text"
          placeholder="value"
          class="w-full rounded border border-gray-700 bg-gray-800 px-2 py-1.5 text-sm text-gray-200 placeholder-gray-600 focus:border-blue-500 focus:outline-none font-mono"
          bind:value={newValue}
          onkeydown={(e) => { if (e.key === "Enter") addCustomVar(); }}
        />
      </div>
      <button
        class="flex-shrink-0 rounded-lg bg-gray-700 px-3 py-1.5 text-sm text-gray-200 hover:bg-gray-600 transition-colors disabled:opacity-50"
        onclick={addCustomVar}
        disabled={!newKey.trim()}
      >
        Add
      </button>
    </div>
  </div>

  <!-- Launch button -->
  <div class="pt-2">
    <button
      class="w-full rounded-lg bg-blue-600 px-4 py-3 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
      onclick={launch}
      disabled={!selectedProject || launching}
    >
      {#if launching}
        Launching...
      {:else}
        Launch Claude Code
      {/if}
    </button>
  </div>

  <!-- Result message -->
  {#if launchResult}
    <div
      class="rounded-lg border px-4 py-3 text-sm {launchIsError
        ? 'border-red-700 bg-red-900/20 text-red-300'
        : 'border-green-700 bg-green-900/20 text-green-300'}"
    >
      {launchResult}
    </div>
  {/if}

</div>

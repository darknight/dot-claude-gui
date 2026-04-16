<script lang="ts">
  import { ipcClient } from "$lib/ipc/client.js";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { pluginsStore } from "$lib/stores/plugins.svelte";
  import { t } from "$lib/i18n";

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
      await ipcClient.launchClaude({
        projectPath: selectedProject.path,
        env,
      });
      launchResult = t("launcher.launchSuccess");
      launchIsError = false;
    } catch (e) {
      launchResult = t("launcher.launchError", { message: e instanceof Error ? e.message : "Launch failed" });
      launchIsError = true;
    } finally {
      launching = false;
    }
  }
</script>

<div class="flex flex-1 flex-col overflow-y-auto p-6 space-y-5">

  <!-- Section heading -->
  <div>
    <h2 class="text-sm font-semibold" style="color: var(--text-primary)">{t("launcher.title")}</h2>
    <p class="mt-1 text-xs" style="color: var(--text-muted)">
      {t("launcher.description")}
    </p>
  </div>

  <!-- Project selector -->
  <div class="space-y-1.5">
    <label for="launcher-project" class="block text-xs font-medium" style="color: var(--text-muted)">
      {t("launcher.projectLabel")}
    </label>
    {#if projectsStore.projects.length === 0}
      <p class="text-xs" style="color: var(--text-muted)">{t("launcher.noProjects")}</p>
    {:else}
      <select
        id="launcher-project"
        class="input-base"
        bind:value={selectedProjectId}
      >
        <option value="">{t("launcher.selectProjectPlaceholder")}</option>
        {#each projectsStore.projects as project (project.id)}
          <option value={project.id}>{project.name}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- Config summary card -->
  {#if selectedProject}
    {@const settings = configStore.userSettings}
    <div class="card space-y-2">
      <h3 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">{t("launcher.configSummary")}</h3>
      <div class="grid grid-cols-3 gap-3">
        <div class="rounded px-3 py-2 text-center" style="background-color: var(--bg-tertiary)">
          <p class="text-xs" style="color: var(--text-muted)">{t("settings.languageLabel")}</p>
          <p class="mt-0.5 text-sm font-medium" style="color: var(--text-primary)">
            {settings.language ?? "default"}
          </p>
        </div>
        <div class="rounded px-3 py-2 text-center" style="background-color: var(--bg-tertiary)">
          <p class="text-xs" style="color: var(--text-muted)">{t("settings.defaultMode")}</p>
          <p class="mt-0.5 text-sm font-medium" style="color: var(--text-primary)">
            {settings.permissions?.defaultMode ?? "ask"}
          </p>
        </div>
        <div class="rounded px-3 py-2 text-center" style="background-color: var(--bg-tertiary)">
          <p class="text-xs" style="color: var(--text-muted)">{t("launcher.activePlugins")}</p>
          <p class="mt-0.5 text-sm font-medium" style="color: var(--text-primary)">{activePluginCount}</p>
        </div>
      </div>
      <p class="text-xs" style="color: var(--text-muted)">Path: {selectedProject.path}</p>
    </div>
  {/if}

  <!-- Environment variables -->
  <div class="space-y-3">
    <h3 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
      {t("settings.environment")}
    </h3>

    <!-- Settings env vars as checkboxes -->
    {#if Object.keys(settingsEnv).length > 0}
      <div class="card space-y-2">
        <p class="text-xs mb-2" style="color: var(--text-muted)">{t("launcher.fromUserSettings")}</p>
        {#each Object.entries(settingsEnv) as [key, value] (key)}
          <label class="flex items-center gap-3 cursor-pointer group">
            <input
              type="checkbox"
              class="h-3.5 w-3.5 rounded"
              style="accent-color: var(--accent-primary)"
              bind:checked={envChecks[key]}
            />
            <span class="flex-1 min-w-0">
              <span class="text-sm font-mono" style="color: var(--text-secondary)">{key}</span>
              <span class="mx-1" style="color: var(--text-muted)">=</span>
              <span class="text-sm font-mono truncate" style="color: var(--text-muted)">{value}</span>
            </span>
          </label>
        {/each}
      </div>
    {:else}
      <p class="text-xs italic" style="color: var(--text-muted)">{t("launcher.noEnvVars")}</p>
    {/if}

    <!-- Custom env vars -->
    {#if customEnv.length > 0}
      <div class="card space-y-2">
        <p class="text-xs mb-2" style="color: var(--text-muted)">{t("launcher.customVariables")}</p>
        {#each customEnv as cv, i (i)}
          <div class="flex items-center gap-2">
            <input
              type="checkbox"
              class="h-3.5 w-3.5 flex-shrink-0 rounded"
              style="accent-color: var(--accent-primary)"
              bind:checked={cv.enabled}
            />
            <span class="font-mono text-sm" style="color: var(--text-secondary)">{cv.key}</span>
            <span style="color: var(--text-muted)">=</span>
            <span class="flex-1 truncate font-mono text-sm" style="color: var(--text-muted)">{cv.value}</span>
            <button
              class="btn-danger-ghost flex-shrink-0"
              onclick={() => removeCustomVar(i)}
            >
              {t("common.remove")}
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Add custom var inputs -->
    <div class="flex items-end gap-2">
      <div class="flex-1 space-y-1">
        <label for="env-key" class="block text-xs" style="color: var(--text-muted)">{t("launcher.keyLabel")}</label>
        <input
          id="env-key"
          type="text"
          placeholder="MY_VAR"
          class="input-base font-mono"
          bind:value={newKey}
          onkeydown={(e) => { if (e.key === "Enter") addCustomVar(); }}
        />
      </div>
      <div class="flex-1 space-y-1">
        <label for="env-value" class="block text-xs" style="color: var(--text-muted)">{t("launcher.valueLabel")}</label>
        <input
          id="env-value"
          type="text"
          placeholder="value"
          class="input-base font-mono"
          bind:value={newValue}
          onkeydown={(e) => { if (e.key === "Enter") addCustomVar(); }}
        />
      </div>
      <button
        class="btn-secondary flex-shrink-0 rounded-lg px-3 py-1.5 text-sm disabled:opacity-50"
        onclick={addCustomVar}
        disabled={!newKey.trim()}
      >
        {t("common.add")}
      </button>
    </div>
  </div>

  <!-- Launch button -->
  <div class="pt-2">
    <button
      class="btn-primary w-full rounded-lg px-4 py-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
      onclick={launch}
      disabled={!selectedProject || launching}
    >
      {#if launching}
        {t("launcher.launching")}
      {:else}
        {t("launcher.launchButton")}
      {/if}
    </button>
  </div>

  <!-- Result message -->
  {#if launchResult}
    <div class="rounded-lg {launchIsError ? 'alert-error' : 'alert-success'} text-sm">
      {launchResult}
    </div>
  {/if}

</div>

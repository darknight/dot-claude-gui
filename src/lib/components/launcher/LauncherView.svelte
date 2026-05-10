<script lang="ts">
  import { ipcClient } from "$lib/ipc/client.js";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { launcherStore } from "$lib/stores/launcher.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { CLAUDE_ENV_VARS } from "$lib/data/claudeEnvVars";
  import { t } from "$lib/i18n";

  let configDirCache = $state<string | null>(null);
  async function getAccountDir(name: string): Promise<string> {
    if (!configDirCache) {
      configDirCache = await ipcClient.getConfigDir();
    }
    return `${configDirCache}/accounts/${name}`;
  }

  const accountDangling = $derived(
    launcherStore.accountName !== undefined &&
      !accountsStore.has(launcherStore.accountName),
  );

  let newKey = $state("");
  let newValue = $state("");
  let newArgFlag = $state("");
  let newArgValue = $state("");
  let launching = $state(false);
  let launchResult = $state<string>("");
  let launchIsError = $state(false);

  const selectedProject = $derived(
    projectsStore.projects.find((p) => p.id === launcherStore.selectedProjectId),
  );

  // Look up arg metadata so the value input can be hidden for boolean flags.
  const newArgMeta = $derived(
    launcherStore.claudeArgs.find((a) => a.flag === newArgFlag),
  );
  const newArgTakesValue = $derived(
    newArgMeta ? newArgMeta.takesValue : true,
  );

  // Reload persisted env/args whenever selection changes.
  $effect(() => {
    if (!selectedProject) return;
    launcherStore.loadForProject(selectedProject.path);
  });

  function addCustomVar() {
    if (!selectedProject) return;
    const key = newKey.trim();
    if (!key) return;
    launcherStore.addCustomVar(selectedProject.path, key, newValue);
    newKey = "";
    newValue = "";
  }

  function addCustomArg() {
    if (!selectedProject) return;
    const flag = newArgFlag.trim();
    if (!flag) return;
    const value = newArgValue.trim() || undefined;
    launcherStore.addCustomArg(selectedProject.path, flag, value);
    newArgFlag = "";
    newArgValue = "";
  }

  async function launch() {
    if (!selectedProject) return;
    launching = true;
    launchResult = "";
    launchIsError = false;
    try {
      const env: Record<string, string> = {};
      for (const cv of launcherStore.customEnv) {
        if (cv.enabled && cv.key) env[cv.key] = cv.value;
      }
      const args: string[] = [];
      // Account: inject CLAUDE_CONFIG_DIR if a valid account is bound; otherwise fall back to ~/.claude/.
      // When an account is bound we also pass `--setting-sources project,local` so claude
      // doesn't load `~/.claude/settings.json` (the "user" scope is anchored to $HOME and
      // would otherwise leak enabledPlugins/extraKnownMarketplaces/env into the account dir).
      if (launcherStore.accountName) {
        if (accountsStore.has(launcherStore.accountName)) {
          env.CLAUDE_CONFIG_DIR = await getAccountDir(launcherStore.accountName);
          args.push("--setting-sources", "project,local");
        } else {
          launchResult = t("launcher.accountFallbackToast", { name: launcherStore.accountName });
          launchIsError = true;
        }
      }
      for (const a of launcherStore.customArgs) {
        if (!a.enabled || !a.flag) continue;
        args.push(a.flag);
        if (a.value !== undefined && a.value !== "") args.push(a.value);
      }
      await ipcClient.launchClaude({
        projectPath: selectedProject.path,
        env,
        args,
        preferredTerminal: appSettingsStore.preferences.preferredTerminal ?? "terminal",
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

  {#if !selectedProject}
    <p class="text-xs" style="color: var(--text-muted)">{t("launcher.selectProjectHint")}</p>
  {:else}
    <!-- Account selection -->
    <div class="space-y-1.5">
      <label for="launcher-account" class="block text-xs font-medium" style="color: var(--text-muted)">
        {t("launcher.accountLabel")}
      </label>
      <select
        id="launcher-account"
        class="input-base"
        value={launcherStore.accountName ?? ""}
        onchange={(e) => {
          const v = (e.target as HTMLSelectElement).value;
          launcherStore.setAccount(selectedProject.path, v || undefined);
        }}
      >
        <option value="">{t("launcher.accountDefault")}</option>
        {#each accountsStore.accounts as a (a.name)}
          <option value={a.name}>{a.name}</option>
        {/each}
        {#if accountDangling}
          <option value={launcherStore.accountName} selected>{t("launcher.accountDeleted", { name: launcherStore.accountName ?? "" })}</option>
        {/if}
      </select>
    </div>

    <!-- Environment variables -->
    <div class="space-y-3">
      <h3 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
        {t("settings.environment")}
      </h3>

      <!-- Custom env vars -->
      {#if launcherStore.customEnv.length > 0}
        <div class="card space-y-2">
          <p class="text-xs mb-2" style="color: var(--text-muted)">{t("launcher.customVariables")}</p>
          {#each launcherStore.customEnv as cv, i (i)}
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                class="h-3.5 w-3.5 flex-shrink-0 rounded"
                style="accent-color: var(--accent-primary)"
                checked={cv.enabled}
                onchange={(e) => launcherStore.setCustomVarEnabled(selectedProject.path, i, (e.target as HTMLInputElement).checked)}
              />
              <span class="font-mono text-sm" style="color: var(--text-secondary)">{cv.key}</span>
              <span style="color: var(--text-muted)">=</span>
              <span class="flex-1 truncate font-mono text-sm" style="color: var(--text-muted)">{cv.value}</span>
              <button
                class="btn-danger-ghost flex-shrink-0"
                onclick={() => launcherStore.removeCustomVar(selectedProject.path, i)}
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
            list="claude-env-keys"
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

    <!-- CLI Arguments -->
    <div class="space-y-3">
      <h3 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
        {t("launcher.argsTitle")}
      </h3>

      {#if launcherStore.claudeArgsLoaded && launcherStore.claudeArgs.length === 0}
        <p class="text-xs italic" style="color: var(--text-muted)">{t("launcher.argsHelpUnavailable")}</p>
      {/if}

      {#if launcherStore.customArgs.length > 0}
        <div class="card space-y-2">
          {#each launcherStore.customArgs as arg, i (i)}
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                class="h-3.5 w-3.5 flex-shrink-0 rounded"
                style="accent-color: var(--accent-primary)"
                checked={arg.enabled}
                onchange={(e) => launcherStore.setArgEnabled(selectedProject.path, i, (e.target as HTMLInputElement).checked)}
              />
              <span class="flex-shrink-0 font-mono text-sm" style="color: var(--text-secondary)">{arg.flag}</span>
              <input
                type="text"
                class="input-base flex-1 font-mono text-sm"
                placeholder={t("launcher.argValuePlaceholder")}
                value={arg.value ?? ""}
                oninput={(e) => launcherStore.setArgValue(selectedProject.path, i, (e.target as HTMLInputElement).value)}
              />
              <button
                class="btn-danger-ghost flex-shrink-0"
                onclick={() => launcherStore.removeCustomArg(selectedProject.path, i)}
              >
                {t("common.remove")}
              </button>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Add arg inputs -->
      <div class="flex items-end gap-2">
        <div class="flex-1 space-y-1">
          <label for="arg-flag" class="block text-xs" style="color: var(--text-muted)">{t("launcher.argFlagPlaceholder")}</label>
          <input
            id="arg-flag"
            type="text"
            list="claude-args"
            placeholder={t("launcher.argFlagPlaceholder")}
            class="input-base font-mono"
            bind:value={newArgFlag}
            onkeydown={(e) => { if (e.key === "Enter") addCustomArg(); }}
          />
        </div>
        {#if newArgTakesValue}
          <div class="flex-1 space-y-1">
            <label for="arg-value" class="block text-xs" style="color: var(--text-muted)">{newArgMeta?.valueHint ?? t("launcher.argValuePlaceholder")}</label>
            <input
              id="arg-value"
              type="text"
              placeholder={newArgMeta?.valueHint ?? t("launcher.argValuePlaceholder")}
              class="input-base font-mono"
              bind:value={newArgValue}
              onkeydown={(e) => { if (e.key === "Enter") addCustomArg(); }}
            />
          </div>
        {/if}
        <button
          class="btn-secondary flex-shrink-0 rounded-lg px-3 py-1.5 text-sm disabled:opacity-50"
          onclick={addCustomArg}
          disabled={!newArgFlag.trim()}
        >
          {t("common.add")}
        </button>
      </div>

      {#if newArgMeta}
        <p class="text-xs" style="color: var(--text-muted)">{newArgMeta.description}</p>
      {/if}
    </div>

    <!-- Launch button -->
    <div class="pt-2">
      <button
        class="btn-primary w-full rounded-lg px-4 py-3 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-50"
        onclick={launch}
        disabled={launching}
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
  {/if}

</div>

<datalist id="claude-env-keys">
  {#each CLAUDE_ENV_VARS as v (v.name)}
    <option value={v.name}>{v.description}</option>
  {/each}
</datalist>
<datalist id="claude-args">
  {#each launcherStore.claudeArgs as a (a.flag)}
    <option value={a.flag}>{a.description}</option>
  {/each}
</datalist>

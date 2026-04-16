<script lang="ts">
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import type { AddMcpServerRequest } from "$lib/api/types";
  import { t } from "$lib/i18n";

  // Form state
  let name = $state("");
  let transport = $state<"stdio" | "sse" | "http">("stdio");
  let command = $state("");
  let argsRaw = $state("");
  let url = $state("");
  let scope = $state<"user" | "project" | "local">("user");

  // Env vars state
  let envEntries = $state<{ key: string; value: string }[]>([]);
  let newEnvKey = $state("");
  let newEnvValue = $state("");
  let envError = $state("");

  // Submission state
  let submitting = $state(false);
  let submitError = $state("");
  let submitSuccess = $state(false);

  function addEnvEntry() {
    const trimmedKey = newEnvKey.trim();
    if (!trimmedKey) {
      envError = t("mcp.errorKeyEmpty");
      return;
    }
    if (envEntries.some((e) => e.key === trimmedKey)) {
      envError = t("mcp.errorKeyExists", { key: trimmedKey });
      return;
    }
    envEntries = [...envEntries, { key: trimmedKey, value: newEnvValue }];
    newEnvKey = "";
    newEnvValue = "";
    envError = "";
  }

  function removeEnvEntry(index: number) {
    envEntries = envEntries.filter((_, i) => i !== index);
  }

  function handleEnvKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") addEnvEntry();
  }

  async function handleSubmit(event: Event) {
    event.preventDefault();
    submitError = "";
    submitSuccess = false;

    if (!name.trim()) {
      submitError = t("mcp.errorNameRequired");
      return;
    }

    if (transport === "stdio" && !command.trim()) {
      submitError = t("mcp.errorCommandRequired");
      return;
    }

    if ((transport === "sse" || transport === "http") && !url.trim()) {
      submitError = t("mcp.errorUrlRequired");
      return;
    }

    const args = argsRaw
      .split(/[\s,]+/)
      .map((a) => a.trim())
      .filter(Boolean);

    const env = envEntries.length > 0
      ? Object.fromEntries(envEntries.map((e) => [e.key, e.value]))
      : undefined;

    const req: AddMcpServerRequest = {
      name: name.trim(),
      transport,
      commandOrUrl: transport === "stdio" ? command.trim() : url.trim(),
      args: args.length > 0 ? args : undefined,
      scope,
      env,
    };

    submitting = true;
    try {
      await mcpStore.addServer(req);
      if (!mcpStore.error) {
        submitSuccess = true;
        // Reset form
        name = "";
        command = "";
        argsRaw = "";
        url = "";
        envEntries = [];
        transport = "stdio";
        scope = "user";
        // Refresh list
        await mcpStore.loadServers();
      } else {
        submitError = mcpStore.error;
      }
    } catch (e) {
      submitError = e instanceof Error ? e.message : t("mcp.errorAddFailed");
    } finally {
      submitting = false;
    }
  }

  const inputClass = "input-base";
  const labelClass = "block text-xs font-medium mb-1";
</script>

<div class="flex-1 overflow-auto p-6">
  <form onsubmit={handleSubmit} class="max-w-xl space-y-5">
    <h2 class="text-sm font-semibold" style="color: var(--text-primary)">{t("mcp.addMcpServer")}</h2>

    {#if submitError}
      <div class="alert-error">
        <p class="text-xs">{submitError}</p>
      </div>
    {/if}

    {#if submitSuccess}
      <div class="alert-success">
        <p class="text-xs">{t("mcp.serverAddedSuccess")}</p>
      </div>
    {/if}

    <!-- Name -->
    <div>
      <label for="mcp-name" class={labelClass} style="color: var(--text-muted)">{t("mcp.nameLabel")}</label>
      <input
        id="mcp-name"
        type="text"
        bind:value={name}
        placeholder="my-server"
        class={inputClass}
        required
      />
    </div>

    <!-- Transport -->
    <div>
      <span class={labelClass} style="color: var(--text-muted)">{t("mcp.transportLabel")}</span>
      <div class="flex gap-4">
        {#each (["stdio", "sse", "http"] as const) as t_val}
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="transport"
              value={t_val}
              bind:group={transport}
              style="accent-color: var(--accent-primary)"
            />
            <span class="text-sm" style="color: var(--text-secondary)">{t_val}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Stdio: command + args -->
    {#if transport === "stdio"}
      <div>
        <label for="mcp-command" class={labelClass} style="color: var(--text-muted)">{t("mcp.commandLabel")}</label>
        <input
          id="mcp-command"
          type="text"
          bind:value={command}
          placeholder="npx"
          class={inputClass}
        />
      </div>
      <div>
        <label for="mcp-args" class={labelClass} style="color: var(--text-muted)">{t("mcp.argumentsLabel")} <span style="color: var(--text-muted)">{t("mcp.argumentsHint")}</span></label>
        <input
          id="mcp-args"
          type="text"
          bind:value={argsRaw}
          placeholder="-y, @modelcontextprotocol/server-filesystem, /tmp"
          class={inputClass}
        />
      </div>
    {:else}
      <!-- SSE / HTTP: URL -->
      <div>
        <label for="mcp-url" class={labelClass} style="color: var(--text-muted)">{t("mcp.urlLabel")}</label>
        <input
          id="mcp-url"
          type="url"
          bind:value={url}
          placeholder="https://example.com/mcp"
          class={inputClass}
        />
      </div>
    {/if}

    <!-- Scope -->
    <div>
      <label for="mcp-scope" class={labelClass} style="color: var(--text-muted)">{t("mcp.scopeLabel")}</label>
      <select
        id="mcp-scope"
        bind:value={scope}
        class="input-base w-auto"
      >
        <option value="user">user</option>
        <option value="project">project</option>
        <option value="local">local</option>
      </select>
    </div>

    <!-- Env vars -->
    <div class="space-y-2">
      <span class={labelClass} style="color: var(--text-muted)">{t("mcp.envVarsLabel")}</span>

      {#if envEntries.length > 0}
        <div class="space-y-1.5">
          {#each envEntries as entry, index}
            <div class="group flex items-center gap-2">
              <code class="code-block shrink-0 px-2 py-1 text-xs font-mono">
                {entry.key}
              </code>
              <span class="text-sm" style="color: var(--text-muted)">=</span>
              <span class="flex-1 text-xs truncate" style="color: var(--text-muted)">{entry.value || '""'}</span>
              <button
                type="button"
                onclick={() => removeEnvEntry(index)}
                class="btn-danger-ghost text-xs opacity-0 group-hover:opacity-100 transition-opacity"
              >
                {t("common.remove")}
              </button>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Add env entry row -->
      <div class="flex items-center gap-2">
        <input
          type="text"
          bind:value={newEnvKey}
          onkeydown={handleEnvKeydown}
          oninput={() => { envError = ""; }}
          placeholder="KEY"
          class="input-base w-32 font-mono text-xs"
        />
        <span class="text-sm" style="color: var(--text-muted)">=</span>
        <input
          type="text"
          bind:value={newEnvValue}
          onkeydown={handleEnvKeydown}
          placeholder="value"
          class="input-base flex-1 text-xs"
        />
        <button
          type="button"
          onclick={addEnvEntry}
          class="btn-secondary shrink-0 text-xs"
        >
          {t("common.add")}
        </button>
      </div>
      {#if envError}
        <p class="text-xs" style="color: var(--status-error-text)">{envError}</p>
      {/if}
    </div>

    <!-- Submit -->
    <div class="pt-2">
      <button
        type="submit"
        disabled={submitting}
        class="btn-primary disabled:opacity-50"
      >
        {submitting ? t("mcp.adding") : t("mcp.addServer")}
      </button>
    </div>
  </form>
</div>

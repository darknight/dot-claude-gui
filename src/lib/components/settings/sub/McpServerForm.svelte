<script lang="ts">
  import type { McpServerRef } from "$lib/api/types";
  import { t } from "$lib/i18n";

  let {
    servers = $bindable(),
    onChange,
  }: {
    servers: McpServerRef[];
    onChange: () => void;
  } = $props();

  let newName = $state("");
  let newUrl = $state("");

  function add() {
    const name = newName.trim();
    if (!name) return;
    servers = [...servers, { serverName: name, serverUrl: newUrl.trim() || undefined }];
    newName = "";
    newUrl = "";
    onChange();
  }

  function remove(i: number) {
    servers = servers.filter((_, idx) => idx !== i);
    onChange();
  }
</script>

<div class="space-y-2">
  <ul class="space-y-1">
    {#each servers as server, i (i + ":" + (server.serverName ?? ""))}
      <li class="flex items-center gap-2 text-sm">
        <span style="color: var(--text-primary)">{server.serverName ?? "(unnamed)"}</span>
        {#if server.serverUrl}
          <span class="font-mono text-xs" style="color: var(--text-muted)">
            {server.serverUrl}
          </span>
        {/if}
        <button type="button"
                onclick={() => remove(i)}
                class="text-xs px-2 py-0.5 rounded"
                style="color: var(--status-error-text); background-color: var(--status-error-bg)">
          ✕
        </button>
      </li>
    {/each}
  </ul>

  <div class="flex gap-2 items-center">
    <input type="text" bind:value={newName}
           placeholder={t("settings.mcpPolicy.serverNamePlaceholder")}
           class="input-base flex-1" />
    <input type="text" bind:value={newUrl}
           placeholder={t("settings.mcpPolicy.serverUrlPlaceholder")}
           class="input-base flex-1" />
    <button type="button" onclick={add}
            class="btn-secondary text-xs px-3 py-1">
      {t("settings.mcpPolicy.addServer")}
    </button>
  </div>
</div>

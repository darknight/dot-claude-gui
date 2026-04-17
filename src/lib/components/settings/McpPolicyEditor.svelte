<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import McpServerForm from "./sub/McpServerForm.svelte";
  import { t } from "$lib/i18n";
  import type { McpServerRef } from "$lib/api/types";

  const settings = $derived(configStore.activeSettings);

  let allowed = $state<McpServerRef[]>(
    (settings.allowedMcpServers as McpServerRef[]) ?? [],
  );
  let denied = $state<McpServerRef[]>(
    (settings.deniedMcpServers as McpServerRef[]) ?? [],
  );
  let enabledJsonText = $state(
    ((settings.enabledMcpjsonServers as string[]) ?? []).join("\n"),
  );
  let disabledJsonText = $state(
    ((settings.disabledMcpjsonServers as string[]) ?? []).join("\n"),
  );
  let enableAllProjectMcpServers = $state(
    (settings.enableAllProjectMcpServers as boolean) ?? false,
  );
  let allowManagedMcpServersOnly = $state(
    (settings.allowManagedMcpServersOnly as boolean) ?? false,
  );

  $effect(() => {
    allowed = (settings.allowedMcpServers as McpServerRef[]) ?? [];
    denied = (settings.deniedMcpServers as McpServerRef[]) ?? [];
    enabledJsonText = ((settings.enabledMcpjsonServers as string[]) ?? []).join("\n");
    disabledJsonText = ((settings.disabledMcpjsonServers as string[]) ?? []).join("\n");
    enableAllProjectMcpServers = (settings.enableAllProjectMcpServers as boolean) ?? false;
    allowManagedMcpServersOnly = (settings.allowManagedMcpServersOnly as boolean) ?? false;
  });

  function parseLines(text: string): string[] | undefined {
    const lines = text.split("\n").map((s) => s.trim()).filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  const previewData = $derived({
    allowedMcpServers: allowed.length === 0 ? undefined : allowed,
    deniedMcpServers: denied.length === 0 ? undefined : denied,
    enabledMcpjsonServers: parseLines(enabledJsonText),
    disabledMcpjsonServers: parseLines(disabledJsonText),
    enableAllProjectMcpServers,
    allowManagedMcpServersOnly,
  });

  function save() {
    configStore.save({
      allowedMcpServers: allowed.length === 0 ? undefined : allowed,
      deniedMcpServers: denied.length === 0 ? undefined : denied,
      enabledMcpjsonServers: parseLines(enabledJsonText),
      disabledMcpjsonServers: parseLines(disabledJsonText),
      enableAllProjectMcpServers,
      allowManagedMcpServersOnly,
    });
  }
</script>

<div class="space-y-6 max-w-2xl">
  <div class="space-y-3">
    <label class="flex items-center gap-3 cursor-pointer">
      <input type="checkbox" bind:checked={enableAllProjectMcpServers}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" style="color: var(--text-secondary)"
            title={t("settings.fields.enableAllProjectMcpServers.tooltip")}>
        {t("settings.fields.enableAllProjectMcpServers.label")}
      </span>
    </label>
    <label class="flex items-center gap-3 cursor-pointer">
      <input type="checkbox" bind:checked={allowManagedMcpServersOnly}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" style="color: var(--text-secondary)"
            title={t("settings.fields.allowManagedMcpServersOnly.tooltip")}>
        {t("settings.fields.allowManagedMcpServersOnly.label")}
      </span>
    </label>
  </div>

  <section>
    <h3 class="text-sm font-semibold mb-2" style="color: var(--text-primary)">
      {t("settings.mcpPolicy.allowed")}
    </h3>
    <McpServerForm bind:servers={allowed} onChange={() => configStore.markDirty()} />
  </section>

  <section>
    <h3 class="text-sm font-semibold mb-2" style="color: var(--text-primary)">
      {t("settings.mcpPolicy.denied")}
    </h3>
    <McpServerForm bind:servers={denied} onChange={() => configStore.markDirty()} />
  </section>

  <section class="space-y-3">
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.mcpPolicy.enabledJson")}
      </label>
      <textarea bind:value={enabledJsonText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.mcpPolicy.jsonServerPlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.mcpPolicy.disabledJson")}
      </label>
      <textarea bind:value={disabledJsonText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.mcpPolicy.jsonServerPlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
  </section>

  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button type="button" onclick={save}
            disabled={!configStore.isDirty || configStore.saving}
            class="btn-primary text-sm px-4 py-2">
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button type="button" onclick={() => configStore.revert()}
            disabled={!configStore.isDirty}
            class="btn-secondary text-sm px-4 py-2">
      {t("common.revert")}
    </button>
  </div>

  <JsonPreview data={previewData} title="MCP JSON" />
</div>

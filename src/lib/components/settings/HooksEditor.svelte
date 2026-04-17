<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import { deepEqual } from "$lib/utils/diff";
  import JsonPreview from "./JsonPreview.svelte";
  import type { HookGroup, HookDefinition } from "$lib/api/types";
  import { t } from "$lib/i18n";

  const validEvents = [
    "PreToolUse", "PostToolUse", "PostToolUseFailure", "Notification", "UserPromptSubmit",
    "SessionStart", "SessionEnd", "Stop", "StopFailure", "SubagentStart", "SubagentStop",
    "PreCompact", "PostCompact", "PermissionRequest", "PermissionDenied", "Setup",
    "TeammateIdle", "TaskCreated", "TaskCompleted", "Elicitation", "ElicitationResult",
    "ConfigChange", "WorktreeCreate", "WorktreeRemove", "InstructionsLoaded",
    "CwdChanged", "FileChanged",
  ];

  let selectedEvent = $state("PreToolUse");
  let hooks = $state<Record<string, HookGroup[]>>({});

  // Policy fields (M7)
  const activeSettings = $derived(configStore.activeSettings);
  let disableAllHooks = $state(
    (activeSettings.disableAllHooks as boolean | undefined) ?? false,
  );
  let allowedHttpHookUrlsText = $state(
    ((activeSettings.allowedHttpHookUrls as string[] | undefined) ?? []).join("\n"),
  );
  let httpHookAllowedEnvVarsText = $state(
    ((activeSettings.httpHookAllowedEnvVars as string[] | undefined) ?? []).join("\n"),
  );
  let allowManagedHooksOnly = $state(
    (activeSettings.allowManagedHooksOnly as boolean | undefined) ?? false,
  );
  let allowManagedPermissionRulesOnly = $state(
    (activeSettings.allowManagedPermissionRulesOnly as boolean | undefined) ?? false,
  );
  let disableSkillShellExecution = $state(
    (activeSettings.disableSkillShellExecution as boolean | undefined) ?? false,
  );

  $effect(() => {
    disableAllHooks =
      (activeSettings.disableAllHooks as boolean | undefined) ?? false;
    allowedHttpHookUrlsText = (
      (activeSettings.allowedHttpHookUrls as string[] | undefined) ?? []
    ).join("\n");
    httpHookAllowedEnvVarsText = (
      (activeSettings.httpHookAllowedEnvVars as string[] | undefined) ?? []
    ).join("\n");
    allowManagedHooksOnly =
      (activeSettings.allowManagedHooksOnly as boolean | undefined) ?? false;
    allowManagedPermissionRulesOnly =
      (activeSettings.allowManagedPermissionRulesOnly as boolean | undefined) ?? false;
    disableSkillShellExecution =
      (activeSettings.disableSkillShellExecution as boolean | undefined) ?? false;
  });

  function parseLines(text: string): string[] | undefined {
    const lines = text.split("\n").map((s) => s.trim()).filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  // Sync from store when settings change.
  // `$state.snapshot` strips the reactive proxy wrapper so the value can be
  // safely deep-cloned — `structuredClone` throws DataCloneError on Svelte 5
  // proxies because they use Symbols/getters it can't serialize.
  $effect(() => {
    const src = configStore.activeSettings.hooks;
    hooks = $state.snapshot(src ?? {}) as Record<string, HookGroup[]>;
  });

  function getGroups(): HookGroup[] {
    return hooks[selectedEvent] ?? [];
  }

  function setGroups(groups: HookGroup[]) {
    hooks = { ...hooks, [selectedEvent]: groups };
    configStore.markDirty();
  }

  function addRule() {
    const groups = getGroups();
    const newGroup: HookGroup = {
      matcher: "",
      hooks: [{ type: "command", command: "" }],
    };
    setGroups([...groups, newGroup]);
  }

  function removeRule(index: number) {
    const groups = getGroups();
    const updated = groups.filter((_, i) => i !== index);
    setGroups(updated);
  }

  function updateMatcher(index: number, value: string) {
    const groups = $state.snapshot(getGroups()) as HookGroup[];
    groups[index] = { ...groups[index], matcher: value };
    setGroups(groups);
  }

  function updateCondition(index: number, value: string) {
    const groups = $state.snapshot(getGroups()) as HookGroup[];
    if (value.trim() === "") {
      const { if: _removed, ...rest } = groups[index];
      groups[index] = rest;
    } else {
      groups[index] = { ...groups[index], if: value };
    }
    setGroups(groups);
  }

  function addHookDefinition(groupIndex: number) {
    const groups = $state.snapshot(getGroups()) as HookGroup[];
    const hookDef: HookDefinition = { type: "command", command: "" };
    groups[groupIndex] = {
      ...groups[groupIndex],
      hooks: [...(groups[groupIndex].hooks ?? []), hookDef],
    };
    setGroups(groups);
  }

  function removeHookDefinition(groupIndex: number, hookIndex: number) {
    const groups = $state.snapshot(getGroups()) as HookGroup[];
    const defs = (groups[groupIndex].hooks ?? []).filter((_, i) => i !== hookIndex);
    groups[groupIndex] = { ...groups[groupIndex], hooks: defs };
    setGroups(groups);
  }

  function updateHookType(groupIndex: number, hookIndex: number, type: "command" | "http") {
    const groups = $state.snapshot(getGroups()) as HookGroup[];
    const defs = [...(groups[groupIndex].hooks ?? [])];
    if (type === "command") {
      defs[hookIndex] = { type: "command", command: "" };
    } else {
      defs[hookIndex] = { type: "http", url: "", method: "POST" };
    }
    groups[groupIndex] = { ...groups[groupIndex], hooks: defs };
    setGroups(groups);
  }

  function updateHookField(
    groupIndex: number,
    hookIndex: number,
    field: keyof HookDefinition,
    value: string | number | undefined,
  ) {
    const groups = $state.snapshot(getGroups()) as HookGroup[];
    const defs = [...(groups[groupIndex].hooks ?? [])];
    defs[hookIndex] = { ...defs[hookIndex], [field]: value };
    groups[groupIndex] = { ...groups[groupIndex], hooks: defs };
    setGroups(groups);
  }

  function eventCount(event: string): number {
    return (hooks[event] ?? []).length;
  }

  function originalGroup(index: number): HookGroup | undefined {
    return configStore.activeSettings.hooks?.[selectedEvent]?.[index];
  }

  function isRuleDirty(index: number, group: HookGroup): boolean {
    return !deepEqual(group, originalGroup(index));
  }

  function isEventDirty(event: string): boolean {
    return !deepEqual(hooks[event] ?? [], configStore.activeSettings.hooks?.[event] ?? []);
  }

  function save() {
    // Filter out events with no groups
    const filtered: Record<string, HookGroup[]> = {};
    for (const [event, groups] of Object.entries(hooks)) {
      if (groups.length > 0) {
        filtered[event] = groups;
      }
    }
    configStore.save({
      hooks: filtered,
      disableAllHooks,
      allowedHttpHookUrls: parseLines(allowedHttpHookUrlsText),
      httpHookAllowedEnvVars: parseLines(httpHookAllowedEnvVarsText),
      allowManagedHooksOnly,
      allowManagedPermissionRulesOnly,
      disableSkillShellExecution,
    });
  }

  const previewData = $derived({ hooks });
</script>

<div class="space-y-5 max-w-2xl">
  <!-- Event Type Selector -->
  <div class="space-y-1">
    <label
      for="eventType"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      {t("settings.eventType")}
      <DirtyDot dirty={isEventDirty(selectedEvent)} />
    </label>
    <select
      id="eventType"
      bind:value={selectedEvent}
      class="input-base"
    >
      {#each validEvents as event}
        {@const count = eventCount(event)}
        {@const dirty = isEventDirty(event)}
        <option value={event}>
          {event}{count > 0 ? ` (${count})` : ""}{dirty ? " ●" : ""}
        </option>
      {/each}
    </select>
  </div>

  <!-- Empty-but-dirty hint: local cleared all rules but disk still has them -->
  {#if getGroups().length === 0 && isEventDirty(selectedEvent)}
    <div class="card text-xs" style="color: var(--text-muted); border-left: 3px solid var(--status-warning-text)">
      {t("settings.hooksEmptyDirtyHint")}
    </div>
  {/if}

  <!-- Hook Rules List -->
  <div class="space-y-3">
    {#each getGroups() as group, groupIndex}
      <div class="card space-y-3">
        <!-- Header row with delete button -->
        <div class="flex items-center justify-between">
          <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)">
            {t("settings.hookRule", { n: groupIndex + 1 })}
            <DirtyDot dirty={isRuleDirty(groupIndex, group)} />
          </span>
          <button
            type="button"
            onclick={() => removeRule(groupIndex)}
            class="btn-danger-ghost text-xs"
          >
            {t("settings.deleteRule")}
          </button>
        </div>

        <!-- Matcher -->
        <div class="space-y-1">
          <label
            for="matcher-{groupIndex}"
            class="block text-xs font-medium" style="color: var(--text-muted)"
          >
            {t("settings.matcher")}
          </label>
          <input
            id="matcher-{groupIndex}"
            type="text"
            value={group.matcher ?? ""}
            oninput={(e) => updateMatcher(groupIndex, (e.target as HTMLInputElement).value)}
            placeholder={t("settings.matcherPlaceholder")}
            class="input-base"
          />
        </div>

        <!-- Optional "if" condition -->
        <div class="space-y-1">
          <label
            for="condition-{groupIndex}"
            class="block text-xs font-medium" style="color: var(--text-muted)"
          >
            {t("settings.condition")}
          </label>
          <input
            id="condition-{groupIndex}"
            type="text"
            value={group.if ?? ""}
            oninput={(e) => updateCondition(groupIndex, (e.target as HTMLInputElement).value)}
            placeholder={t("settings.conditionPlaceholder")}
            class="input-base"
          />
        </div>

        <!-- Hook Definitions -->
        <div class="space-y-2">
          <span class="block text-xs font-medium" style="color: var(--text-muted)">
            {t("settings.hooksLabel")}
          </span>
          {#each (group.hooks ?? []) as hookDef, hookIndex}
            <div class="ml-3 pl-3 border-l-2 space-y-2" style="border-color: var(--border-color)">
              <!-- Type toggle + remove -->
              <div class="flex items-center gap-4">
                <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                  <input
                    type="radio"
                    name="hooktype-{groupIndex}-{hookIndex}"
                    value="command"
                    checked={hookDef.type === "command"}
                    onchange={() => updateHookType(groupIndex, hookIndex, "command")}
                    style="accent-color: var(--accent-primary)"
                  />
                  <span style="color: var(--text-secondary)">{t("settings.hookTypeCommand")}</span>
                </label>
                <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                  <input
                    type="radio"
                    name="hooktype-{groupIndex}-{hookIndex}"
                    value="http"
                    checked={hookDef.type === "http"}
                    onchange={() => updateHookType(groupIndex, hookIndex, "http")}
                    style="accent-color: var(--accent-primary)"
                  />
                  <span style="color: var(--text-secondary)">{t("settings.hookTypeHttp")}</span>
                </label>
                {#if (group.hooks ?? []).length > 1}
                  <button
                    type="button"
                    onclick={() => removeHookDefinition(groupIndex, hookIndex)}
                    class="btn-danger-ghost ml-auto text-xs"
                  >
                    {t("common.remove")}
                  </button>
                {/if}
              </div>

              <!-- Command fields -->
              {#if hookDef.type === "command" || !hookDef.type}
                <div class="space-y-1">
                  <label
                    for="command-{groupIndex}-{hookIndex}"
                    class="block text-xs" style="color: var(--text-muted)"
                  >
                    {t("settings.commandLabel")}
                  </label>
                  <input
                    id="command-{groupIndex}-{hookIndex}"
                    type="text"
                    value={hookDef.command ?? ""}
                    oninput={(e) =>
                      updateHookField(groupIndex, hookIndex, "command", (e.target as HTMLInputElement).value)}
                    placeholder={t("settings.commandPlaceholder")}
                    class="input-base"
                  />
                </div>
              {/if}

              <!-- HTTP fields -->
              {#if hookDef.type === "http"}
                <div class="space-y-2">
                  <div class="space-y-1">
                    <label
                      for="url-{groupIndex}-{hookIndex}"
                      class="block text-xs" style="color: var(--text-muted)"
                    >
                      {t("settings.urlLabel")}
                    </label>
                    <input
                      id="url-{groupIndex}-{hookIndex}"
                      type="text"
                      value={hookDef.url ?? ""}
                      oninput={(e) =>
                        updateHookField(groupIndex, hookIndex, "url", (e.target as HTMLInputElement).value)}
                      placeholder={t("settings.urlPlaceholder")}
                      class="input-base"
                    />
                  </div>
                  <div class="flex gap-3">
                    <div class="space-y-1 flex-1">
                      <label
                        for="method-{groupIndex}-{hookIndex}"
                        class="block text-xs" style="color: var(--text-muted)"
                      >
                        {t("settings.methodLabel")}
                      </label>
                      <select
                        id="method-{groupIndex}-{hookIndex}"
                        value={hookDef.method ?? "POST"}
                        onchange={(e) =>
                          updateHookField(groupIndex, hookIndex, "method", (e.target as HTMLSelectElement).value)}
                        class="input-base"
                      >
                        <option value="GET">GET</option>
                        <option value="POST">POST</option>
                        <option value="PUT">PUT</option>
                      </select>
                    </div>
                    <div class="space-y-1 w-32">
                      <label
                        for="timeout-{groupIndex}-{hookIndex}"
                        class="block text-xs" style="color: var(--text-muted)"
                      >
                        {t("settings.timeoutLabel")}
                      </label>
                      <input
                        id="timeout-{groupIndex}-{hookIndex}"
                        type="number"
                        min="0"
                        value={hookDef.timeout ?? ""}
                        oninput={(e) => {
                          const v = (e.target as HTMLInputElement).value;
                          updateHookField(
                            groupIndex,
                            hookIndex,
                            "timeout",
                            v === "" ? undefined : Number(v),
                          );
                        }}
                        placeholder="5000"
                        class="input-base"
                      />
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          {/each}

          <!-- Add hook definition button -->
          <button
            type="button"
            onclick={() => addHookDefinition(groupIndex)}
            class="ml-3 text-xs" style="color: var(--accent-text)"
          >
            {t("settings.addHook")}
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Add Hook Rule button -->
  <button
    type="button"
    onclick={addRule}
    class="btn-secondary text-sm px-3 py-1.5" style="border-style: dashed"
  >
    {t("settings.addHookRule")}
  </button>

  <!-- Policy (M7) -->
  <section class="border-t pt-4 mt-4" style="border-color: var(--border-color)">
    <h3 class="text-sm font-semibold mb-3" style="color: var(--text-primary)">
      {t("settings.hooks.policy")}
    </h3>

    <label class="flex items-center gap-3 cursor-pointer mb-2">
      <input type="checkbox" bind:checked={disableAllHooks}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.disableAllHooks.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.disableAllHooks.label")}
      </span>
    </label>

    <label class="flex items-center gap-3 cursor-pointer mb-2">
      <input type="checkbox" bind:checked={allowManagedHooksOnly}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.allowManagedHooksOnly.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.allowManagedHooksOnly.label")}
      </span>
    </label>

    <label class="flex items-center gap-3 cursor-pointer mb-2">
      <input type="checkbox" bind:checked={allowManagedPermissionRulesOnly}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.allowManagedPermissionRulesOnly.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.allowManagedPermissionRulesOnly.label")}
      </span>
    </label>

    <label class="flex items-center gap-3 cursor-pointer mb-3">
      <input type="checkbox" bind:checked={disableSkillShellExecution}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.disableSkillShellExecution.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.disableSkillShellExecution.label")}
      </span>
    </label>

    <div class="space-y-1 mb-3">
      <label class="block text-sm font-medium" style="color: var(--text-secondary)"
             title={t("settings.fields.allowedHttpHookUrls.tooltip")}>
        {t("settings.fields.allowedHttpHookUrls.label")}
      </label>
      <textarea bind:value={allowedHttpHookUrlsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder="https://hooks.internal.corp"
                class="input-base font-mono text-xs"></textarea>
    </div>

    <div class="space-y-1">
      <label class="block text-sm font-medium" style="color: var(--text-secondary)"
             title={t("settings.fields.httpHookAllowedEnvVars.tooltip")}>
        {t("settings.fields.httpHookAllowedEnvVars.label")}
      </label>
      <textarea bind:value={httpHookAllowedEnvVarsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder="GITHUB_TOKEN"
                class="input-base font-mono text-xs"></textarea>
    </div>
  </section>

  <!-- Save / Revert -->
  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button
      type="button"
      onclick={save}
      disabled={!configStore.isDirty || configStore.saving}
      class="btn-primary text-sm px-4 py-2"
    >
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button
      type="button"
      onclick={() => configStore.revert()}
      disabled={!configStore.isDirty}
      class="btn-secondary text-sm px-4 py-2"
    >
      {t("common.revert")}
    </button>
  </div>

  <!-- JSON Preview -->
  <JsonPreview data={previewData} title={t("settings.hooksJson")} />
</div>

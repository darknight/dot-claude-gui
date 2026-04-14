<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import type { HookGroup, HookDefinition } from "$lib/api/types";

  const validEvents = [
    "PreToolUse", "PostToolUse", "PostToolUseFailure", "Notification", "UserPromptSubmit",
    "SessionStart", "SessionEnd", "Stop", "StopFailure", "SubagentStart", "SubagentStop",
    "PreCompact", "PostCompact", "PermissionRequest", "PermissionDenied", "Setup",
    "TeammateIdle", "TaskCreated", "TaskCompleted", "Elicitation", "ElicitationResult",
    "ConfigChange", "WorktreeCreate", "WorktreeRemove", "InstructionsLoaded",
    "CwdChanged", "FileChanged",
  ];

  const settings = $derived(configStore.activeSettings);

  let selectedEvent = $state("PreToolUse");
  let hooks = $state<Record<string, HookGroup[]>>({});

  // Sync from store when settings change
  $effect(() => {
    hooks = structuredClone(settings.hooks ?? {});
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
    const groups = structuredClone(getGroups());
    groups[index] = { ...groups[index], matcher: value };
    setGroups(groups);
  }

  function updateCondition(index: number, value: string) {
    const groups = structuredClone(getGroups());
    if (value.trim() === "") {
      const { if: _removed, ...rest } = groups[index];
      groups[index] = rest;
    } else {
      groups[index] = { ...groups[index], if: value };
    }
    setGroups(groups);
  }

  function addHookDefinition(groupIndex: number) {
    const groups = structuredClone(getGroups());
    const hookDef: HookDefinition = { type: "command", command: "" };
    groups[groupIndex] = {
      ...groups[groupIndex],
      hooks: [...(groups[groupIndex].hooks ?? []), hookDef],
    };
    setGroups(groups);
  }

  function removeHookDefinition(groupIndex: number, hookIndex: number) {
    const groups = structuredClone(getGroups());
    const defs = (groups[groupIndex].hooks ?? []).filter((_, i) => i !== hookIndex);
    groups[groupIndex] = { ...groups[groupIndex], hooks: defs };
    setGroups(groups);
  }

  function updateHookType(groupIndex: number, hookIndex: number, type: "command" | "http") {
    const groups = structuredClone(getGroups());
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
    const groups = structuredClone(getGroups());
    const defs = [...(groups[groupIndex].hooks ?? [])];
    defs[hookIndex] = { ...defs[hookIndex], [field]: value };
    groups[groupIndex] = { ...groups[groupIndex], hooks: defs };
    setGroups(groups);
  }

  function eventCount(event: string): number {
    return (hooks[event] ?? []).length;
  }

  function save() {
    // Filter out events with no groups
    const filtered: Record<string, HookGroup[]> = {};
    for (const [event, groups] of Object.entries(hooks)) {
      if (groups.length > 0) {
        filtered[event] = groups;
      }
    }
    configStore.save({ hooks: filtered });
  }

  const previewData = $derived({ hooks });
</script>

<div class="space-y-5 max-w-2xl">
  <!-- Event Type Selector -->
  <div class="space-y-1">
    <label
      for="eventType"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300"
    >
      Event Type
    </label>
    <select
      id="eventType"
      bind:value={selectedEvent}
      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      {#each validEvents as event}
        {@const count = eventCount(event)}
        <option value={event}>
          {event}{count > 0 ? ` (${count})` : ""}
        </option>
      {/each}
    </select>
  </div>

  <!-- Hook Rules List -->
  <div class="space-y-3">
    {#each getGroups() as group, groupIndex}
      <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-3 space-y-3">
        <!-- Header row with delete button -->
        <div class="flex items-center justify-between">
          <span class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
            Rule {groupIndex + 1}
          </span>
          <button
            type="button"
            onclick={() => removeRule(groupIndex)}
            class="text-xs text-red-500 hover:text-red-700 dark:hover:text-red-400"
          >
            Delete Rule
          </button>
        </div>

        <!-- Matcher -->
        <div class="space-y-1">
          <label
            for="matcher-{groupIndex}"
            class="block text-xs font-medium text-gray-600 dark:text-gray-400"
          >
            Matcher
          </label>
          <input
            id="matcher-{groupIndex}"
            type="text"
            value={group.matcher ?? ""}
            oninput={(e) => updateMatcher(groupIndex, (e.target as HTMLInputElement).value)}
            placeholder="e.g. Bash or * to match all"
            class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <!-- Optional "if" condition -->
        <div class="space-y-1">
          <label
            for="condition-{groupIndex}"
            class="block text-xs font-medium text-gray-600 dark:text-gray-400"
          >
            Condition (optional)
          </label>
          <input
            id="condition-{groupIndex}"
            type="text"
            value={group.if ?? ""}
            oninput={(e) => updateCondition(groupIndex, (e.target as HTMLInputElement).value)}
            placeholder="e.g. tool_name == 'Bash'"
            class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <!-- Hook Definitions -->
        <div class="space-y-2">
          <span class="block text-xs font-medium text-gray-600 dark:text-gray-400">
            Hooks
          </span>
          {#each (group.hooks ?? []) as hookDef, hookIndex}
            <div class="ml-3 pl-3 border-l-2 border-gray-200 dark:border-gray-600 space-y-2">
              <!-- Type toggle + remove -->
              <div class="flex items-center gap-4">
                <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                  <input
                    type="radio"
                    name="hooktype-{groupIndex}-{hookIndex}"
                    value="command"
                    checked={hookDef.type === "command"}
                    onchange={() => updateHookType(groupIndex, hookIndex, "command")}
                    class="accent-blue-500"
                  />
                  <span class="text-gray-700 dark:text-gray-300">Command</span>
                </label>
                <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                  <input
                    type="radio"
                    name="hooktype-{groupIndex}-{hookIndex}"
                    value="http"
                    checked={hookDef.type === "http"}
                    onchange={() => updateHookType(groupIndex, hookIndex, "http")}
                    class="accent-blue-500"
                  />
                  <span class="text-gray-700 dark:text-gray-300">HTTP</span>
                </label>
                {#if (group.hooks ?? []).length > 1}
                  <button
                    type="button"
                    onclick={() => removeHookDefinition(groupIndex, hookIndex)}
                    class="ml-auto text-xs text-red-400 hover:text-red-600"
                  >
                    Remove
                  </button>
                {/if}
              </div>

              <!-- Command fields -->
              {#if hookDef.type === "command" || !hookDef.type}
                <div class="space-y-1">
                  <label
                    for="command-{groupIndex}-{hookIndex}"
                    class="block text-xs text-gray-500 dark:text-gray-400"
                  >
                    Command
                  </label>
                  <input
                    id="command-{groupIndex}-{hookIndex}"
                    type="text"
                    value={hookDef.command ?? ""}
                    oninput={(e) =>
                      updateHookField(groupIndex, hookIndex, "command", (e.target as HTMLInputElement).value)}
                    placeholder="/path/to/script.sh"
                    class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              {/if}

              <!-- HTTP fields -->
              {#if hookDef.type === "http"}
                <div class="space-y-2">
                  <div class="space-y-1">
                    <label
                      for="url-{groupIndex}-{hookIndex}"
                      class="block text-xs text-gray-500 dark:text-gray-400"
                    >
                      URL
                    </label>
                    <input
                      id="url-{groupIndex}-{hookIndex}"
                      type="text"
                      value={hookDef.url ?? ""}
                      oninput={(e) =>
                        updateHookField(groupIndex, hookIndex, "url", (e.target as HTMLInputElement).value)}
                      placeholder="https://example.com/webhook"
                      class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <div class="flex gap-3">
                    <div class="space-y-1 flex-1">
                      <label
                        for="method-{groupIndex}-{hookIndex}"
                        class="block text-xs text-gray-500 dark:text-gray-400"
                      >
                        Method
                      </label>
                      <select
                        id="method-{groupIndex}-{hookIndex}"
                        value={hookDef.method ?? "POST"}
                        onchange={(e) =>
                          updateHookField(groupIndex, hookIndex, "method", (e.target as HTMLSelectElement).value)}
                        class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                      >
                        <option value="GET">GET</option>
                        <option value="POST">POST</option>
                        <option value="PUT">PUT</option>
                      </select>
                    </div>
                    <div class="space-y-1 w-32">
                      <label
                        for="timeout-{groupIndex}-{hookIndex}"
                        class="block text-xs text-gray-500 dark:text-gray-400"
                      >
                        Timeout (ms)
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
                        class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1.5 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
            class="ml-3 text-xs text-blue-500 hover:text-blue-700 dark:hover:text-blue-400"
          >
            + Add Hook
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Add Hook Rule button -->
  <button
    type="button"
    onclick={addRule}
    class="text-sm px-3 py-1.5 border border-dashed border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-600 dark:text-gray-400"
  >
    + Add Hook Rule
  </button>

  <!-- Save / Revert -->
  <div class="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
    <button
      type="button"
      onclick={save}
      disabled={!configStore.isDirty || configStore.saving}
      class="text-sm px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40"
    >
      {configStore.saving ? "Saving..." : "Save"}
    </button>
    <button
      type="button"
      onclick={() => configStore.revert()}
      disabled={!configStore.isDirty}
      class="text-sm px-4 py-2 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40"
    >
      Revert
    </button>
  </div>

  <!-- JSON Preview -->
  <JsonPreview data={previewData} title="Hooks (JSON)" />
</div>

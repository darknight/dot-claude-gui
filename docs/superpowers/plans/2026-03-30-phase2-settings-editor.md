# Phase 2: Settings Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Settings module with form-based sub-editors for each settings section (General, Permissions, Hooks, Sandbox, Environment, Status Line), per-project scope support, JSON preview, and save/revert functionality.

**Architecture:** Each settings section gets its own Svelte component that receives the current settings as a prop and dispatches change events. A parent SettingsEditor orchestrates which sub-editor is shown, manages save/revert state, and communicates with the config store. The existing daemon PUT /api/v1/config/user endpoint handles merging and validation.

**Tech Stack:** Svelte 5 (runes: $state, $derived, $effect, $props, $bindable), TypeScript, Tailwind CSS 4

**Prerequisite:** Phase 1 complete (all 38 tests passing, frontend building)

---

## File Structure (Phase 2)

```
src/lib/components/
├── settings/
│   ├── SettingsEditor.svelte      # Orchestrator: sub-editor routing + save/revert
│   ├── GeneralEditor.svelte       # language, thinking, updates, etc.
│   ├── PermissionsEditor.svelte   # allow/deny/ask lists + defaultMode
│   ├── HooksEditor.svelte         # Dynamic hook rule editor
│   ├── SandboxEditor.svelte       # Sandbox path lists + toggles
│   ├── EnvVarEditor.svelte        # Key-value env var table
│   ├── StatusLineEditor.svelte    # Status line config
│   └── JsonPreview.svelte         # Read-only JSON preview panel
├── shared/
│   ├── ConnectionStatus.svelte    # (exists)
│   ├── ProjectSelector.svelte     # (new) Project dropdown for header
│   ├── StringListEditor.svelte    # (new) Reusable add/remove string list
│   └── ScopeSelector.svelte       # (new) User/Project scope toggle
```

Files to modify:
- `src/App.svelte` — wire SettingsEditor into the Settings module view
- `src/lib/stores/config.svelte.ts` — add project config support, dirty tracking
- `src/lib/api/client.ts` — add `updateProjectConfig` method

---

### Task 1: Shared Components — StringListEditor, ScopeSelector, JsonPreview

**Files:**
- Create: `src/lib/components/shared/StringListEditor.svelte`
- Create: `src/lib/components/shared/ScopeSelector.svelte`
- Create: `src/lib/components/settings/JsonPreview.svelte`

- [ ] **Step 1: Create StringListEditor component**

A reusable component for editing arrays of strings (used by Permissions allow/deny/ask, Sandbox paths, etc.)

```svelte
<!-- src/lib/components/shared/StringListEditor.svelte -->
<script lang="ts">
  let { items = $bindable([]), placeholder = "Add item...", label = "" }: {
    items: string[];
    placeholder?: string;
    label?: string;
  } = $props();

  let newItem = $state("");

  function addItem() {
    const trimmed = newItem.trim();
    if (trimmed && !items.includes(trimmed)) {
      items = [...items, trimmed];
      newItem = "";
    }
  }

  function removeItem(index: number) {
    items = items.filter((_, i) => i !== index);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      addItem();
    }
  }
</script>

{#if label}
  <label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1.5">{label}</label>
{/if}

<div class="space-y-1.5">
  {#each items as item, index}
    <div class="flex items-center gap-2 group">
      <code class="flex-1 text-sm bg-gray-50 dark:bg-gray-800 px-2.5 py-1.5 rounded border border-gray-200 dark:border-gray-700">
        {item}
      </code>
      <button
        type="button"
        onclick={() => removeItem(index)}
        class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600 text-sm transition-opacity"
        title="Remove"
      >
        &times;
      </button>
    </div>
  {/each}

  <div class="flex gap-2">
    <input
      type="text"
      bind:value={newItem}
      onkeydown={handleKeydown}
      {placeholder}
      class="flex-1 text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
    <button
      type="button"
      onclick={addItem}
      disabled={!newItem.trim()}
      class="text-sm px-3 py-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40 disabled:cursor-not-allowed"
    >
      Add
    </button>
  </div>
</div>
```

- [ ] **Step 2: Create ScopeSelector component**

```svelte
<!-- src/lib/components/shared/ScopeSelector.svelte -->
<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte";

  let { scope = $bindable("user") }: { scope: string } = $props();
</script>

<div class="flex items-center gap-3 text-sm">
  <label class="text-gray-500 dark:text-gray-400">Scope:</label>
  <select
    bind:value={scope}
    class="bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
  >
    <option value="user">User</option>
    {#if projectsStore.activeProject}
      <option value="project">{projectsStore.activeProject.name}</option>
    {/if}
  </select>
</div>
```

- [ ] **Step 3: Create JsonPreview component**

```svelte
<!-- src/lib/components/settings/JsonPreview.svelte -->
<script lang="ts">
  let { data, title = "JSON Preview" }: { data: unknown; title?: string } = $props();

  let collapsed = $state(false);
  const json = $derived(JSON.stringify(data, null, 2));
</script>

<div class="mt-6 border-t border-gray-200 dark:border-gray-700 pt-4">
  <button
    type="button"
    onclick={() => (collapsed = !collapsed)}
    class="flex items-center gap-2 text-xs font-medium text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 mb-2"
  >
    <span class="transition-transform" class:rotate-90={!collapsed}>&#9654;</span>
    {title}
  </button>

  {#if !collapsed}
    <pre class="text-xs bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded p-3 overflow-auto max-h-64 font-mono">{json}</pre>
  {/if}
</div>
```

- [ ] **Step 4: Verify frontend builds**

Run: `pnpm build`
Expected: Build succeeds

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/shared/StringListEditor.svelte src/lib/components/shared/ScopeSelector.svelte src/lib/components/settings/JsonPreview.svelte
git commit -m "feat: add shared form components (StringListEditor, ScopeSelector, JsonPreview)"
```

---

### Task 2: Config Store Enhancements — Dirty Tracking + Project Config

**Files:**
- Modify: `src/lib/stores/config.svelte.ts`
- Modify: `src/lib/api/client.ts`

- [ ] **Step 1: Add updateProjectConfig to API client**

Add this method to the `DaemonClient` class in `src/lib/api/client.ts`:

```typescript
  async updateProjectConfig(projectId: string, settings: Partial<Settings>): Promise<ConfigResponse> {
    return this.fetch(`/api/v1/config/project/${projectId}`, {
      method: "PUT",
      body: JSON.stringify({ settings }),
    });
  }
```

- [ ] **Step 2: Enhance config store with dirty tracking and section editing**

Replace `src/lib/stores/config.svelte.ts` with:

```typescript
// src/lib/stores/config.svelte.ts
import { connectionStore } from "./connection.svelte";
import { projectsStore } from "./projects.svelte";
import type { Settings } from "$lib/api/types";

class ConfigStore {
  userSettings = $state<Settings>({});
  projectSettings = $state<Settings>({});
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  /** The settings being edited (user or project depending on active scope) */
  get activeSettings(): Settings {
    return this.activeScope === "project" ? this.projectSettings : this.userSettings;
  }

  /** Draft state for the section currently being edited */
  draft = $state<Record<string, unknown>>({});
  activeScope = $state<"user" | "project">("user");
  isDirty = $state(false);

  async loadUserConfig() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      const res = await client.getUserConfig();
      this.userSettings = res.settings;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load config";
    } finally {
      this.loading = false;
    }
  }

  async loadProjectConfig(projectId: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      const res = await client.getProjectConfig(projectId);
      this.projectSettings = res.settings;
    } catch {
      this.projectSettings = {};
    } finally {
      this.loading = false;
    }
  }

  /** Start editing: snapshot the current section into draft */
  startEditing(sectionKey: string, sectionData: unknown) {
    this.draft = { [sectionKey]: structuredClone(sectionData) };
    this.isDirty = false;
  }

  /** Mark draft as changed */
  markDirty() {
    this.isDirty = true;
  }

  /** Save the current draft to the appropriate scope */
  async save(partialSettings: Partial<Settings>) {
    const client = connectionStore.client;
    if (!client) return;
    this.saving = true;
    this.error = "";
    try {
      if (this.activeScope === "project" && projectsStore.activeProjectId) {
        const res = await client.updateProjectConfig(
          projectsStore.activeProjectId,
          partialSettings,
        );
        this.projectSettings = res.settings;
      } else {
        const res = await client.updateUserConfig(partialSettings);
        this.userSettings = res.settings;
      }
      this.isDirty = false;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save";
      throw e;
    } finally {
      this.saving = false;
    }
  }

  /** Revert draft to current saved state */
  revert() {
    this.isDirty = false;
    this.draft = {};
  }
}

export const configStore = new ConfigStore();
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/config.svelte.ts src/lib/api/client.ts
git commit -m "feat: enhance config store with dirty tracking and project config support"
```

---

### Task 3: SettingsEditor Orchestrator + App.svelte Wiring

**Files:**
- Create: `src/lib/components/settings/SettingsEditor.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create SettingsEditor orchestrator**

This component manages which sub-editor is displayed and the save/revert controls.

```svelte
<!-- src/lib/components/settings/SettingsEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import ScopeSelector from "$lib/components/shared/ScopeSelector.svelte";
  import GeneralEditor from "./GeneralEditor.svelte";
  import PermissionsEditor from "./PermissionsEditor.svelte";
  import HooksEditor from "./HooksEditor.svelte";
  import SandboxEditor from "./SandboxEditor.svelte";
  import EnvVarEditor from "./EnvVarEditor.svelte";
  import StatusLineEditor from "./StatusLineEditor.svelte";

  let { activeSection = "general" }: { activeSection: string } = $props();
</script>

<div class="h-full flex flex-col">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700 shrink-0">
    <ScopeSelector bind:scope={configStore.activeScope} />
    <div class="flex items-center gap-2">
      {#if configStore.isDirty}
        <span class="text-xs text-amber-500">Unsaved changes</span>
      {/if}
      <button
        type="button"
        onclick={() => configStore.revert()}
        disabled={!configStore.isDirty}
        class="text-xs px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40"
      >
        Revert
      </button>
      <button
        type="button"
        onclick={() => {
          // Each sub-editor provides its save data via the configStore.draft
          // This is handled per-editor
        }}
        disabled={!configStore.isDirty || configStore.saving}
        class="text-xs px-3 py-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40"
      >
        {configStore.saving ? "Saving..." : "Save"}
      </button>
    </div>
  </div>

  <!-- Error display -->
  {#if configStore.error}
    <div class="mx-4 mt-2 p-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-xs rounded border border-red-200 dark:border-red-800">
      {configStore.error}
    </div>
  {/if}

  <!-- Sub-editor -->
  <div class="flex-1 overflow-auto p-4">
    {#if activeSection === "general"}
      <GeneralEditor />
    {:else if activeSection === "permissions"}
      <PermissionsEditor />
    {:else if activeSection === "hooks"}
      <HooksEditor />
    {:else if activeSection === "sandbox"}
      <SandboxEditor />
    {:else if activeSection === "environment"}
      <EnvVarEditor />
    {:else if activeSection === "statusline"}
      <StatusLineEditor />
    {/if}
  </div>
</div>
```

- [ ] **Step 2: Create placeholder sub-editors so the build works**

Create these 6 minimal placeholder files:

`src/lib/components/settings/GeneralEditor.svelte`:
```svelte
<p class="text-gray-400 text-sm">General editor — Task 4</p>
```

`src/lib/components/settings/PermissionsEditor.svelte`:
```svelte
<p class="text-gray-400 text-sm">Permissions editor — Task 5</p>
```

`src/lib/components/settings/HooksEditor.svelte`:
```svelte
<p class="text-gray-400 text-sm">Hooks editor — Task 6</p>
```

`src/lib/components/settings/SandboxEditor.svelte`:
```svelte
<p class="text-gray-400 text-sm">Sandbox editor — Task 7</p>
```

`src/lib/components/settings/EnvVarEditor.svelte`:
```svelte
<p class="text-gray-400 text-sm">Environment editor — Task 8</p>
```

`src/lib/components/settings/StatusLineEditor.svelte`:
```svelte
<p class="text-gray-400 text-sm">Status Line editor — Task 9</p>
```

- [ ] **Step 3: Update App.svelte to wire in SettingsEditor**

Modify `src/App.svelte`. The key changes:
1. Import `SettingsEditor`
2. Add a `settingsSection` state variable for sub-navigation
3. When `currentModule === "settings"`, render sub-nav items (General, Permissions, Hooks, Sandbox, Environment, Status Line) in the middle panel
4. When `currentModule === "settings"`, render `<SettingsEditor activeSection={settingsSection} />` in the detail panel instead of the raw JSON

The settings sub-navigation items in the middle panel:
```typescript
const settingsSections = [
  { id: "general", label: "General" },
  { id: "permissions", label: "Permissions" },
  { id: "hooks", label: "Hooks" },
  { id: "sandbox", label: "Sandbox" },
  { id: "environment", label: "Environment" },
  { id: "statusline", label: "Status Line" },
];
```

When a sub-nav item is clicked, set `settingsSection` to its id. Highlight the active one.

In the detail panel, replace the raw JSON dump with:
```svelte
{#if currentModule === "settings"}
  <SettingsEditor activeSection={settingsSection} />
{:else}
  <p class="text-gray-500">Module "{currentModule}" coming in Phase 3-4.</p>
{/if}
```

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/settings/ src/App.svelte
git commit -m "feat: add SettingsEditor orchestrator with sub-navigation and placeholder editors"
```

---

### Task 4: GeneralEditor

**Files:**
- Modify: `src/lib/components/settings/GeneralEditor.svelte`

- [ ] **Step 1: Implement GeneralEditor**

```svelte
<!-- src/lib/components/settings/GeneralEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);

  let language = $state(settings.language ?? "");
  let alwaysThinkingEnabled = $state(settings.alwaysThinkingEnabled ?? false);
  let autoUpdatesChannel = $state(settings.autoUpdatesChannel ?? "stable");
  let minimumVersion = $state(settings.minimumVersion ?? "");
  let includeCoAuthoredBy = $state(settings.includeCoAuthoredBy ?? false);
  let skipDangerousModePermissionPrompt = $state(settings.skipDangerousModePermissionPrompt ?? false);

  // Sync from store when settings change externally
  $effect(() => {
    language = settings.language ?? "";
    alwaysThinkingEnabled = settings.alwaysThinkingEnabled ?? false;
    autoUpdatesChannel = settings.autoUpdatesChannel ?? "stable";
    minimumVersion = settings.minimumVersion ?? "";
    includeCoAuthoredBy = settings.includeCoAuthoredBy ?? false;
    skipDangerousModePermissionPrompt = settings.skipDangerousModePermissionPrompt ?? false;
  });

  function onChange() {
    configStore.markDirty();
  }

  const previewData = $derived({
    language: language || undefined,
    alwaysThinkingEnabled,
    autoUpdatesChannel,
    minimumVersion: minimumVersion || undefined,
    includeCoAuthoredBy,
    skipDangerousModePermissionPrompt,
  });

  // Expose save data
  export async function save() {
    await configStore.save({
      language: language || undefined,
      alwaysThinkingEnabled,
      autoUpdatesChannel,
      minimumVersion: minimumVersion || undefined,
      includeCoAuthoredBy,
      skipDangerousModePermissionPrompt,
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <h2 class="text-base font-semibold">General Settings</h2>

  <!-- Language -->
  <div>
    <label class="block text-sm font-medium mb-1" for="language">Language</label>
    <input
      id="language"
      type="text"
      bind:value={language}
      oninput={onChange}
      placeholder="e.g. Simplified Chinese, en-US"
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <!-- Always Thinking -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={alwaysThinkingEnabled}
      onchange={onChange}
      class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
    />
    <span class="text-sm">Always Thinking Enabled</span>
  </label>

  <!-- Auto Updates Channel -->
  <div>
    <label class="block text-sm font-medium mb-1" for="updates-channel">Auto Updates Channel</label>
    <select
      id="updates-channel"
      bind:value={autoUpdatesChannel}
      onchange={onChange}
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    >
      <option value="stable">stable</option>
      <option value="latest">latest</option>
    </select>
  </div>

  <!-- Minimum Version -->
  <div>
    <label class="block text-sm font-medium mb-1" for="min-version">Minimum Version</label>
    <input
      id="min-version"
      type="text"
      bind:value={minimumVersion}
      oninput={onChange}
      placeholder="e.g. 2.1.63"
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <!-- Include Co-Authored-By -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={includeCoAuthoredBy}
      onchange={onChange}
      class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
    />
    <span class="text-sm">Include Co-Authored-By in Commits</span>
  </label>

  <!-- Skip Dangerous Mode Prompt -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={skipDangerousModePermissionPrompt}
      onchange={onChange}
      class="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
    />
    <span class="text-sm">Skip Dangerous Mode Permission Prompt</span>
  </label>

  <JsonPreview data={previewData} title="General JSON Preview" />
</div>
```

- [ ] **Step 2: Update SettingsEditor save button to call the active editor's save**

The save mechanism needs adjustment. Instead of each editor exporting a `save()` function (which is awkward with Svelte 5), use a simpler approach: each editor calls `configStore.save()` directly when the user clicks save. Update `SettingsEditor.svelte`'s save button:

```svelte
<button
  type="button"
  onclick={handleSave}
  disabled={!configStore.isDirty || configStore.saving}
  class="text-xs px-3 py-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40"
>
  {configStore.saving ? "Saving..." : "Save"}
</button>
```

Add the `handleSave` function in the script block — it reads current form state from `configStore.draft` and calls save. But a simpler pattern: let each sub-editor maintain its own state and have a local save function. The SettingsEditor's Save button dispatches a custom event that the active editor listens to.

**Simplest approach:** Each editor has its own Save/Revert buttons at the bottom. Remove save/revert from the toolbar and let each editor own its workflow. This is simpler and avoids cross-component communication.

Update SettingsEditor to remove the save/revert from toolbar:

```svelte
<!-- SettingsEditor.svelte - simplified toolbar -->
<div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700 shrink-0">
  <ScopeSelector bind:scope={configStore.activeScope} />
  {#if configStore.isDirty}
    <span class="text-xs text-amber-500">Unsaved changes</span>
  {/if}
</div>
```

Add save/revert buttons to GeneralEditor at the bottom:

```svelte
<!-- At the bottom of GeneralEditor, before JsonPreview -->
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
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/settings/
git commit -m "feat: implement GeneralEditor with save/revert and JSON preview"
```

---

### Task 5: PermissionsEditor

**Files:**
- Modify: `src/lib/components/settings/PermissionsEditor.svelte`

- [ ] **Step 1: Implement PermissionsEditor**

```svelte
<!-- src/lib/components/settings/PermissionsEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import StringListEditor from "$lib/components/shared/StringListEditor.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);
  const perms = $derived(settings.permissions);

  let allow = $state<string[]>([]);
  let deny = $state<string[]>([]);
  let ask = $state<string[]>([]);
  let defaultMode = $state("plan");

  // Sync from store
  $effect(() => {
    allow = [...(perms?.allow ?? [])];
    deny = [...(perms?.deny ?? [])];
    ask = [...(perms?.ask ?? [])];
    defaultMode = perms?.defaultMode ?? "plan";
  });

  function onChange() {
    configStore.markDirty();
  }

  // Watch for list changes (StringListEditor uses $bindable)
  $effect(() => {
    // Reading these triggers reactivity when they change
    void allow.length;
    void deny.length;
    void ask.length;
  });

  const validModes = ["acceptEdits", "bypassPermissions", "default", "dontAsk", "plan", "auto"];

  const previewData = $derived({
    allow,
    deny,
    ask,
    defaultMode,
  });

  async function save() {
    await configStore.save({
      permissions: { allow, deny, ask, defaultMode },
    });
  }
</script>

<div class="space-y-6 max-w-xl">
  <h2 class="text-base font-semibold">Permissions</h2>

  <!-- Default Mode -->
  <div>
    <label class="block text-sm font-medium mb-1" for="default-mode">Default Mode</label>
    <select
      id="default-mode"
      bind:value={defaultMode}
      onchange={onChange}
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    >
      {#each validModes as mode}
        <option value={mode}>{mode}</option>
      {/each}
    </select>
  </div>

  <!-- Allow -->
  <div>
    <StringListEditor
      bind:items={allow}
      label="Allow"
      placeholder='e.g. Bash(git:*), WebSearch'
    />
  </div>

  <!-- Deny -->
  <div>
    <StringListEditor
      bind:items={deny}
      label="Deny"
      placeholder='e.g. Bash(rm:*)'
    />
  </div>

  <!-- Ask -->
  <div>
    <StringListEditor
      bind:items={ask}
      label="Ask (require confirmation)"
      placeholder='e.g. Bash(docker:*)'
    />
  </div>

  <!-- Save/Revert -->
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

  <JsonPreview data={previewData} title="Permissions JSON Preview" />
</div>
```

- [ ] **Step 2: Verify build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/settings/PermissionsEditor.svelte
git commit -m "feat: implement PermissionsEditor with allow/deny/ask lists and mode selector"
```

---

### Task 6: HooksEditor (Complex)

**Files:**
- Modify: `src/lib/components/settings/HooksEditor.svelte`

- [ ] **Step 1: Implement HooksEditor**

This is the most complex editor. It needs:
- Event type selector dropdown
- Dynamic list of hook rules per event type
- Each rule: matcher input, hook type toggle (command/http), type-specific fields
- Add/remove rules
- Optional `if` condition field

```svelte
<!-- src/lib/components/settings/HooksEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import type { HookGroup, HookDefinition } from "$lib/api/types";

  const settings = $derived(configStore.activeSettings);

  const validEvents = [
    "PreToolUse", "PostToolUse", "Notification", "Stop", "SubagentStop",
    "CwdChanged", "FileChanged", "ConfigChange", "StopFailure", "TaskCreated",
    "WorktreeCreate", "WorktreeRemove", "PostCompact", "Elicitation", "InstructionsLoaded",
  ];

  let selectedEvent = $state("PreToolUse");
  let hooks = $state<Record<string, HookGroup[]>>({});

  // Sync from store
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
      matcher: "*",
      hooks: [{ type: "command", command: "" }],
    };
    setGroups([...groups, newGroup]);
  }

  function removeRule(index: number) {
    setGroups(getGroups().filter((_, i) => i !== index));
  }

  function updateMatcher(index: number, value: string) {
    const groups = getGroups();
    groups[index] = { ...groups[index], matcher: value };
    setGroups([...groups]);
  }

  function updateCondition(index: number, value: string) {
    const groups = getGroups();
    groups[index] = { ...groups[index], if: value || undefined };
    setGroups([...groups]);
  }

  function updateHookType(groupIndex: number, hookIndex: number, newType: "command" | "http") {
    const groups = getGroups();
    const hook = groups[groupIndex].hooks[hookIndex];
    const updated: HookDefinition = newType === "command"
      ? { type: "command", command: hook.command ?? "" }
      : { type: "http", url: hook.url ?? "", method: hook.method ?? "POST" };
    groups[groupIndex].hooks[hookIndex] = updated;
    setGroups([...groups]);
  }

  function updateHookField(groupIndex: number, hookIndex: number, field: string, value: string | number) {
    const groups = getGroups();
    (groups[groupIndex].hooks[hookIndex] as Record<string, unknown>)[field] = value;
    setGroups([...groups]);
  }

  // Count events with rules for indicator
  const eventCounts = $derived(
    Object.fromEntries(validEvents.map((e) => [e, (hooks[e] ?? []).length]))
  );

  const previewData = $derived({ [selectedEvent]: hooks[selectedEvent] ?? [] });

  async function save() {
    // Only save non-empty events
    const cleanHooks: Record<string, HookGroup[]> = {};
    for (const [event, groups] of Object.entries(hooks)) {
      if (groups.length > 0) {
        cleanHooks[event] = groups;
      }
    }
    await configStore.save({ hooks: cleanHooks });
  }
</script>

<div class="space-y-4 max-w-2xl">
  <h2 class="text-base font-semibold">Hooks</h2>

  <!-- Event Type Selector -->
  <div>
    <label class="block text-sm font-medium mb-1">Event Type</label>
    <select
      bind:value={selectedEvent}
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    >
      {#each validEvents as event}
        <option value={event}>
          {event} {eventCounts[event] ? `(${eventCounts[event]})` : ""}
        </option>
      {/each}
    </select>
  </div>

  <!-- Hook Rules for selected event -->
  <div class="space-y-3">
    {#each getGroups() as group, groupIndex}
      <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-3 space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium text-gray-400">Rule {groupIndex + 1}</span>
          <button
            type="button"
            onclick={() => removeRule(groupIndex)}
            class="text-xs text-red-400 hover:text-red-600"
          >
            Delete
          </button>
        </div>

        <!-- Matcher -->
        <div>
          <label class="block text-xs text-gray-500 mb-1">Matcher</label>
          <input
            type="text"
            value={group.matcher}
            oninput={(e) => updateMatcher(groupIndex, (e.target as HTMLInputElement).value)}
            placeholder="e.g. Bash, Edit, *"
            class="w-full text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- Condition (if) -->
        <div>
          <label class="block text-xs text-gray-500 mb-1">Condition (optional)</label>
          <input
            type="text"
            value={group.if ?? ""}
            oninput={(e) => updateCondition(groupIndex, (e.target as HTMLInputElement).value)}
            placeholder="e.g. tool_input.command matches 'git.*'"
            class="w-full text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- Hook definitions -->
        {#each group.hooks as hook, hookIndex}
          <div class="ml-3 pl-3 border-l-2 border-gray-200 dark:border-gray-700 space-y-2">
            <!-- Type toggle -->
            <div class="flex items-center gap-4">
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input
                  type="radio"
                  name="hook-type-{groupIndex}-{hookIndex}"
                  value="command"
                  checked={hook.type === "command"}
                  onchange={() => updateHookType(groupIndex, hookIndex, "command")}
                  class="text-blue-500"
                />
                Command
              </label>
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input
                  type="radio"
                  name="hook-type-{groupIndex}-{hookIndex}"
                  value="http"
                  checked={hook.type === "http"}
                  onchange={() => updateHookType(groupIndex, hookIndex, "http")}
                  class="text-blue-500"
                />
                HTTP
              </label>
            </div>

            {#if hook.type === "command"}
              <div>
                <label class="block text-xs text-gray-500 mb-1">Command</label>
                <input
                  type="text"
                  value={hook.command ?? ""}
                  oninput={(e) => updateHookField(groupIndex, hookIndex, "command", (e.target as HTMLInputElement).value)}
                  placeholder="/usr/local/bin/my-hook"
                  class="w-full text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
                />
              </div>
            {:else}
              <div>
                <label class="block text-xs text-gray-500 mb-1">URL</label>
                <input
                  type="text"
                  value={hook.url ?? ""}
                  oninput={(e) => updateHookField(groupIndex, hookIndex, "url", (e.target as HTMLInputElement).value)}
                  placeholder="https://hooks.example.com/endpoint"
                  class="w-full text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
                />
              </div>
              <div class="flex gap-3">
                <div class="flex-1">
                  <label class="block text-xs text-gray-500 mb-1">Method</label>
                  <select
                    value={hook.method ?? "POST"}
                    onchange={(e) => updateHookField(groupIndex, hookIndex, "method", (e.target as HTMLSelectElement).value)}
                    class="w-full text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900"
                  >
                    <option value="GET">GET</option>
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                  </select>
                </div>
                <div class="flex-1">
                  <label class="block text-xs text-gray-500 mb-1">Timeout (ms)</label>
                  <input
                    type="number"
                    value={hook.timeout ?? 30000}
                    oninput={(e) => updateHookField(groupIndex, hookIndex, "timeout", parseInt((e.target as HTMLInputElement).value) || 30000)}
                    class="w-full text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900"
                  />
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/each}

    <button
      type="button"
      onclick={addRule}
      class="w-full text-sm py-2 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg text-gray-400 hover:text-gray-600 hover:border-gray-400 transition-colors"
    >
      + Add Hook Rule
    </button>
  </div>

  <!-- Save/Revert -->
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

  <JsonPreview data={previewData} title="Hooks JSON Preview ({selectedEvent})" />
</div>
```

- [ ] **Step 2: Verify build**

Run: `pnpm build`
Expected: Succeeds

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/settings/HooksEditor.svelte
git commit -m "feat: implement HooksEditor with dynamic rule management, command/HTTP toggle, conditions"
```

---

### Task 7: SandboxEditor

**Files:**
- Modify: `src/lib/components/settings/SandboxEditor.svelte`

- [ ] **Step 1: Implement SandboxEditor**

```svelte
<!-- src/lib/components/settings/SandboxEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import StringListEditor from "$lib/components/shared/StringListEditor.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);
  const sandbox = $derived(settings.sandbox);

  let allowRead = $state<string[]>([]);
  let denyRead = $state<string[]>([]);
  let allowWrite = $state<string[]>([]);
  let excludedCommands = $state<string[]>([]);
  let failIfUnavailable = $state(false);
  let enableWeakerNetworkIsolation = $state(false);

  $effect(() => {
    allowRead = [...(sandbox?.allowRead ?? [])];
    denyRead = [...(sandbox?.denyRead ?? [])];
    allowWrite = [...(sandbox?.allowWrite ?? [])];
    excludedCommands = [...(sandbox?.excludedCommands ?? [])];
    failIfUnavailable = sandbox?.failIfUnavailable ?? false;
    enableWeakerNetworkIsolation = sandbox?.enableWeakerNetworkIsolation ?? false;
  });

  function onChange() {
    configStore.markDirty();
  }

  const previewData = $derived({
    allowRead: allowRead.length ? allowRead : undefined,
    denyRead: denyRead.length ? denyRead : undefined,
    allowWrite: allowWrite.length ? allowWrite : undefined,
    excludedCommands: excludedCommands.length ? excludedCommands : undefined,
    failIfUnavailable,
    enableWeakerNetworkIsolation,
  });

  async function save() {
    await configStore.save({
      sandbox: {
        allowRead: allowRead.length ? allowRead : undefined,
        denyRead: denyRead.length ? denyRead : undefined,
        allowWrite: allowWrite.length ? allowWrite : undefined,
        excludedCommands: excludedCommands.length ? excludedCommands : undefined,
        failIfUnavailable,
        enableWeakerNetworkIsolation,
      },
    });
  }
</script>

<div class="space-y-6 max-w-xl">
  <h2 class="text-base font-semibold">Sandbox</h2>

  <StringListEditor bind:items={allowRead} label="Allow Read" placeholder="/path/to/allow" />
  <StringListEditor bind:items={denyRead} label="Deny Read" placeholder="/path/to/deny" />
  <StringListEditor bind:items={allowWrite} label="Allow Write" placeholder="/path/to/allow-write" />
  <StringListEditor bind:items={excludedCommands} label="Excluded Commands" placeholder="e.g. docker" />

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={failIfUnavailable} onchange={onChange} class="w-4 h-4 rounded border-gray-300 text-blue-500" />
    <span class="text-sm">Fail if Sandbox Unavailable</span>
  </label>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={enableWeakerNetworkIsolation} onchange={onChange} class="w-4 h-4 rounded border-gray-300 text-blue-500" />
    <span class="text-sm">Enable Weaker Network Isolation</span>
  </label>

  <div class="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
    <button type="button" onclick={save} disabled={!configStore.isDirty || configStore.saving} class="text-sm px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40">
      {configStore.saving ? "Saving..." : "Save"}
    </button>
    <button type="button" onclick={() => configStore.revert()} disabled={!configStore.isDirty} class="text-sm px-4 py-2 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40">
      Revert
    </button>
  </div>

  <JsonPreview data={previewData} title="Sandbox JSON Preview" />
</div>
```

- [ ] **Step 2: Verify build and commit**

```bash
pnpm build
git add src/lib/components/settings/SandboxEditor.svelte
git commit -m "feat: implement SandboxEditor with path lists and toggles"
```

---

### Task 8: EnvVarEditor

**Files:**
- Modify: `src/lib/components/settings/EnvVarEditor.svelte`

- [ ] **Step 1: Implement EnvVarEditor**

```svelte
<!-- src/lib/components/settings/EnvVarEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);

  let entries = $state<{ key: string; value: string }[]>([]);
  let newKey = $state("");
  let newValue = $state("");

  $effect(() => {
    const env = settings.env ?? {};
    entries = Object.entries(env).map(([key, value]) => ({ key, value }));
  });

  function addEntry() {
    const key = newKey.trim();
    if (!key) return;
    if (entries.some((e) => e.key === key)) return;
    entries = [...entries, { key, value: newValue }];
    newKey = "";
    newValue = "";
    configStore.markDirty();
  }

  function removeEntry(index: number) {
    entries = entries.filter((_, i) => i !== index);
    configStore.markDirty();
  }

  function updateValue(index: number, value: string) {
    entries[index] = { ...entries[index], value };
    entries = [...entries];
    configStore.markDirty();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      addEntry();
    }
  }

  const envObject = $derived(
    Object.fromEntries(entries.map((e) => [e.key, e.value]))
  );

  async function save() {
    await configStore.save({ env: envObject });
  }
</script>

<div class="space-y-4 max-w-2xl">
  <h2 class="text-base font-semibold">Environment Variables</h2>

  <!-- Existing entries -->
  <div class="space-y-2">
    {#each entries as entry, index}
      <div class="flex items-center gap-2 group">
        <code class="text-sm bg-gray-100 dark:bg-gray-800 px-2 py-1.5 rounded min-w-[200px] border border-gray-200 dark:border-gray-700">
          {entry.key}
        </code>
        <span class="text-gray-400">=</span>
        <input
          type="text"
          value={entry.value}
          oninput={(e) => updateValue(index, (e.target as HTMLInputElement).value)}
          class="flex-1 text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
        <button
          type="button"
          onclick={() => removeEntry(index)}
          class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-600 text-sm transition-opacity"
        >
          &times;
        </button>
      </div>
    {/each}
  </div>

  <!-- Add new -->
  <div class="flex items-center gap-2">
    <input
      type="text"
      bind:value={newKey}
      onkeydown={handleKeydown}
      placeholder="KEY"
      class="text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500 w-48 font-mono"
    />
    <span class="text-gray-400">=</span>
    <input
      type="text"
      bind:value={newValue}
      onkeydown={handleKeydown}
      placeholder="value"
      class="flex-1 text-sm px-2.5 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
    <button
      type="button"
      onclick={addEntry}
      disabled={!newKey.trim()}
      class="text-sm px-3 py-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40"
    >
      Add
    </button>
  </div>

  <div class="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
    <button type="button" onclick={save} disabled={!configStore.isDirty || configStore.saving} class="text-sm px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40">
      {configStore.saving ? "Saving..." : "Save"}
    </button>
    <button type="button" onclick={() => configStore.revert()} disabled={!configStore.isDirty} class="text-sm px-4 py-2 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40">
      Revert
    </button>
  </div>

  <JsonPreview data={envObject} title="Environment JSON Preview" />
</div>
```

- [ ] **Step 2: Verify build and commit**

```bash
pnpm build
git add src/lib/components/settings/EnvVarEditor.svelte
git commit -m "feat: implement EnvVarEditor with key-value table and add/remove"
```

---

### Task 9: StatusLineEditor

**Files:**
- Modify: `src/lib/components/settings/StatusLineEditor.svelte`

- [ ] **Step 1: Implement StatusLineEditor**

```svelte
<!-- src/lib/components/settings/StatusLineEditor.svelte -->
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";

  const settings = $derived(configStore.activeSettings);
  const sl = $derived(settings.statusLine);

  let statusType = $state("command");
  let command = $state("");
  let padding = $state(0);

  $effect(() => {
    statusType = sl?.type ?? "command";
    command = sl?.command ?? "";
    padding = sl?.padding ?? 0;
  });

  function onChange() {
    configStore.markDirty();
  }

  const previewData = $derived({
    type: statusType,
    command: command || undefined,
    padding,
  });

  async function save() {
    await configStore.save({
      statusLine: {
        type: statusType,
        command: command || undefined,
        padding,
      },
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <h2 class="text-base font-semibold">Status Line</h2>

  <div>
    <label class="block text-sm font-medium mb-1" for="sl-type">Type</label>
    <select
      id="sl-type"
      bind:value={statusType}
      onchange={onChange}
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    >
      <option value="command">command</option>
    </select>
  </div>

  <div>
    <label class="block text-sm font-medium mb-1" for="sl-command">Command</label>
    <input
      id="sl-command"
      type="text"
      bind:value={command}
      oninput={onChange}
      placeholder="e.g. bunx -y ccstatusline@latest"
      class="w-full text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500 font-mono"
    />
  </div>

  <div>
    <label class="block text-sm font-medium mb-1" for="sl-padding">Padding</label>
    <input
      id="sl-padding"
      type="number"
      bind:value={padding}
      oninput={onChange}
      min="0"
      class="w-32 text-sm px-3 py-2 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <div class="flex gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
    <button type="button" onclick={save} disabled={!configStore.isDirty || configStore.saving} class="text-sm px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40">
      {configStore.saving ? "Saving..." : "Save"}
    </button>
    <button type="button" onclick={() => configStore.revert()} disabled={!configStore.isDirty} class="text-sm px-4 py-2 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40">
      Revert
    </button>
  </div>

  <JsonPreview data={previewData} title="Status Line JSON Preview" />
</div>
```

- [ ] **Step 2: Verify build and commit**

```bash
pnpm build
git add src/lib/components/settings/StatusLineEditor.svelte
git commit -m "feat: implement StatusLineEditor with command and padding fields"
```

---

### Task 10: Backend — PUT Project Config Endpoint

**Files:**
- Modify: `crates/claude-daemon/src/api/config.rs`
- Modify: `crates/claude-daemon/src/server.rs`

The frontend already has `updateProjectConfig()` in the API client (Task 2), but the daemon doesn't have a PUT endpoint for project config yet.

- [ ] **Step 1: Add put_project_config handler**

Add to `crates/claude-daemon/src/api/config.rs`:

```rust
/// PUT /api/v1/config/project/:project_id
pub async fn put_project_config(
    Extension(state): Extension<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    let projects = state.inner.projects.read().await;
    let project = projects.iter().find(|p| p.id == project_id).ok_or_else(|| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: format!("project '{}' not found", project_id),
            details: None,
        }))
    })?;

    let settings_path = project.path.join(".claude").join("settings.json");
    let project_path_str = project.path.to_string_lossy().to_string();
    drop(projects);

    // Read current project settings
    let current = claude_config::parse::read_settings(&settings_path).unwrap_or_default();

    // Parse incoming update
    let update: Settings = serde_json::from_value(body.settings).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: format!("invalid settings: {}", e),
            details: None,
        }))
    })?;

    // Merge
    let merged = claude_config::merge::merge_layers(&[
        claude_config::merge::ConfigLayer { source: ConfigSource::Project, settings: current },
        claude_config::merge::ConfigLayer { source: ConfigSource::Project, settings: update },
    ]);
    let new_settings = merged.settings;

    // Validate
    let errors = claude_config::validate::validate_settings(&new_settings);
    if !errors.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse {
            error: "validation failed".to_string(),
            details: Some(errors),
        })));
    }

    // Ensure .claude directory exists
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Atomic write
    claude_config::write::write_settings(&settings_path, &new_settings).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: e.to_string(),
            details: None,
        }))
    })?;

    Ok(Json(ConfigResponse {
        scope: "project".to_string(),
        settings: new_settings,
        project_path: Some(project_path_str),
    }))
}
```

NOTE: Read the existing `put_user_config` implementation first and follow the same pattern. Adjust import names based on what's actually in the code (the function might use `Extension` or `State` extractor — match what exists).

- [ ] **Step 2: Add route to server.rs**

Add to the protected routes in `server.rs`:
```rust
.route("/api/v1/config/project/{project_id}", put(api::config::put_project_config))
```

This needs to be added alongside the existing GET route for the same path. In axum 0.8, you can chain methods:
```rust
.route("/api/v1/config/project/{project_id}",
    get(api::config::get_project_config).put(api::config::put_project_config))
```

- [ ] **Step 3: Add integration test**

Add to `crates/claude-daemon/tests/api_test.rs`:

```rust
#[tokio::test]
async fn update_project_config() {
    let (dir, token, port, _handle) = start_test_daemon().await;
    let client = Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Create a project directory with .claude/
    let project_dir = dir.path().join("test-project");
    std::fs::create_dir_all(project_dir.join(".claude")).unwrap();
    std::fs::write(
        project_dir.join(".claude").join("settings.json"),
        r#"{"permissions":{"defaultMode":"plan"}}"#,
    ).unwrap();

    // Register project
    let res = client
        .post(format!("{}/api/v1/projects", base))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "path": project_dir.to_str().unwrap() }))
        .send().await.unwrap();
    let project: serde_json::Value = res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap();

    // Update project config
    let res = client
        .put(format!("{}/api/v1/config/project/{}", base, project_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "settings": { "language": "zh-CN" }
        }))
        .send().await.unwrap();
    assert_eq!(res.status(), 200);

    // Verify file on disk
    let content = std::fs::read_to_string(project_dir.join(".claude").join("settings.json")).unwrap();
    let saved: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(saved["language"], "zh-CN");
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p claude-daemon --test api_test`
Expected: All 7 tests pass (6 existing + 1 new)

- [ ] **Step 5: Commit**

```bash
git add crates/claude-daemon/
git commit -m "feat: add PUT /api/v1/config/project endpoint with integration test"
```

---

### Task 11: Full Build Verification

- [ ] **Step 1: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All tests pass

- [ ] **Step 2: Run frontend build**

Run: `pnpm build`
Expected: Succeeds with no errors

- [ ] **Step 3: Verify Tauri compiles**

Run: `cargo build -p dot-claude-gui`
Expected: Compiles

- [ ] **Step 4: Commit any remaining changes**

```bash
git status
# If any uncommitted changes exist, commit them
```

---

## Phase 2 Completion Checklist

- [ ] SettingsEditor orchestrator with sub-navigation
- [ ] GeneralEditor: language, thinking, updates, version, co-authored-by, dangerous mode
- [ ] PermissionsEditor: allow/deny/ask lists with add/remove, defaultMode selector
- [ ] HooksEditor: event type selector, dynamic rule management, command/HTTP toggle, conditions
- [ ] SandboxEditor: path lists for read/write, toggle switches
- [ ] EnvVarEditor: key-value table with add/remove/edit
- [ ] StatusLineEditor: type, command, padding
- [ ] Each editor has JSON preview and save/revert buttons
- [ ] ScopeSelector for user/project scope
- [ ] StringListEditor reusable component
- [ ] PUT project config backend endpoint + test
- [ ] All Rust tests passing
- [ ] Frontend building

**What comes next:**
- **Phase 3:** Plugins (marketplace + install + per-project), Skills, Memory modules
- **Phase 4:** MCP Servers, Effective Config viewer, Project Launcher, App Settings

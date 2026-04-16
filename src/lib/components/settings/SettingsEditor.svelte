<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import ScopeSelector from "$lib/components/shared/ScopeSelector.svelte";
  import GeneralEditor from "./GeneralEditor.svelte";
  import PermissionsEditor from "./PermissionsEditor.svelte";
  import HooksEditor from "./HooksEditor.svelte";
  import SandboxEditor from "./SandboxEditor.svelte";
  import EnvVarEditor from "./EnvVarEditor.svelte";
  import StatusLineEditor from "./StatusLineEditor.svelte";
  import { t } from "$lib/i18n";

  let { activeSection = "general" }: { activeSection: string } = $props();
</script>

<!-- Toolbar -->
<div class="flex items-center justify-between border-b px-4 py-2" style="border-color: var(--border-color); background-color: var(--bg-secondary)">
  <ScopeSelector />
  {#if configStore.isDirty}
    <span class="text-xs" style="color: var(--status-warning-text)">{t("common.unsavedChanges")}</span>
  {/if}
</div>

<!-- Error display -->
{#if configStore.error}
  <div class="border-b px-4 py-2" style="border-color: var(--status-error-text); background-color: var(--status-error-bg)">
    <p class="text-xs" style="color: var(--status-error-text)">{configStore.error}</p>
  </div>
{/if}

<!-- Sub-editor content -->
<div class="flex-1 overflow-auto p-6">
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
  {:else}
    <p class="text-sm" style="color: var(--text-muted)">Unknown section: {activeSection}</p>
  {/if}
</div>

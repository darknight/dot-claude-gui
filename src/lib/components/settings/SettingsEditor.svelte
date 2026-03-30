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

<!-- Toolbar -->
<div class="flex items-center justify-between border-b border-gray-800 bg-gray-900 px-4 py-2">
  <ScopeSelector bind:scope={configStore.activeScope} />
  {#if configStore.isDirty}
    <span class="text-xs text-yellow-400">Unsaved changes</span>
  {/if}
</div>

<!-- Error display -->
{#if configStore.error}
  <div class="border-b border-red-800 bg-red-950 px-4 py-2">
    <p class="text-xs text-red-400">{configStore.error}</p>
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
    <p class="text-sm text-gray-600">Unknown section: {activeSection}</p>
  {/if}
</div>

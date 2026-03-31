<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte";

  let {
    scope = $bindable("user"),
  }: {
    scope: string;
  } = $props();
</script>

<div class="flex items-center gap-2">
  <label class="text-xs font-medium text-gray-400 uppercase tracking-wide">Scope</label>
  <select
    bind:value={scope}
    class="rounded bg-gray-800 border border-gray-700 px-2 py-1 text-sm text-gray-200 focus:outline-none focus:border-blue-500"
  >
    <option value="user">User</option>
    {#if projectsStore.activeProject}
      <option value="project">Project: {projectsStore.activeProject.name}</option>
    {/if}
  </select>
</div>

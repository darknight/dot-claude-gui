<script lang="ts">
  import ClaudeMdList from "$lib/components/claudemd/ClaudeMdList.svelte";
  import ClaudeMdModule from "$lib/components/claudemd/ClaudeMdModule.svelte";
</script>

<div class="flex flex-1 overflow-hidden">
  <aside
    class="w-64 flex-shrink-0 overflow-hidden"
    style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)"
  >
    <ClaudeMdList />
  </aside>
  <div class="flex-1 flex flex-col overflow-hidden">
    <ClaudeMdModule />
  </div>
</div>

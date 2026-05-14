<script lang="ts">
  import MemoryList from "$lib/components/memory/MemoryList.svelte";
  import MemoryModule from "$lib/components/memory/MemoryModule.svelte";
</script>

<div class="flex flex-1 overflow-hidden">
  <aside
    class="w-64 flex-shrink-0 overflow-hidden"
    style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)"
  >
    <MemoryList />
  </aside>
  <div class="flex-1 flex flex-col overflow-hidden">
    <MemoryModule />
  </div>
</div>

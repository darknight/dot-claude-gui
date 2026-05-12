<script lang="ts">
  import SkillList from "$lib/components/skills/SkillList.svelte";
  import SkillsModule from "$lib/components/skills/SkillsModule.svelte";
</script>

<div class="flex flex-1 overflow-hidden">
  <aside
    class="w-64 flex-shrink-0 overflow-hidden"
    style="background-color: var(--bg-secondary); border-right: 1px solid var(--border-color)"
  >
    <SkillList />
  </aside>
  <div class="flex-1 overflow-hidden">
    <SkillsModule />
  </div>
</div>

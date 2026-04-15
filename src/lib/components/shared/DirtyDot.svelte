<script lang="ts">
  let {
    dirty = false,
    title = "Modified",
  }: {
    dirty?: boolean;
    title?: string;
  } = $props();
</script>

{#if dirty}
  <span
    class="inline-block h-1.5 w-1.5 flex-shrink-0 rounded-full bg-orange-500 align-middle ml-1.5"
    aria-label={title}
  ></span>
{/if}

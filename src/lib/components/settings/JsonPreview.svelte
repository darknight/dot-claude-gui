<script lang="ts">
  let {
    data,
    title = "JSON Preview",
  }: {
    data: unknown;
    title?: string;
  } = $props();

  let expanded = $state(false);

  const jsonString = $derived(JSON.stringify(data, null, 2));
</script>

<div class="rounded border border-gray-700 overflow-hidden">
  <button
    onclick={() => (expanded = !expanded)}
    class="flex w-full items-center gap-2 bg-gray-800 px-3 py-2 text-left text-xs font-medium text-gray-400 hover:bg-gray-700 transition-colors"
  >
    <span class="text-gray-500">{expanded ? "▼" : "▶"}</span>
    <span>{title}</span>
  </button>

  {#if expanded}
    <pre class="overflow-auto bg-gray-900 p-3 text-xs text-green-400 font-mono leading-relaxed">{jsonString}</pre>
  {/if}
</div>

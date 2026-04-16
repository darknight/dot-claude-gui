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

<div class="rounded overflow-hidden border" style="border-color: var(--border-color)">
  <button
    onclick={() => (expanded = !expanded)}
    class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs font-medium transition-colors"
    style="background-color: var(--bg-tertiary); color: var(--text-muted)"
  >
    <span>{expanded ? "▼" : "▶"}</span>
    <span>{title}</span>
  </button>

  {#if expanded}
    <pre class="code-block overflow-auto leading-relaxed">{jsonString}</pre>
  {/if}
</div>

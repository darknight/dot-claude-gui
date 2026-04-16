<script lang="ts">
  import DirtyDot from "./DirtyDot.svelte";

  let {
    items = $bindable([]),
    placeholder = "Add item...",
    label,
    dirty = false,
  }: {
    items: string[];
    placeholder?: string;
    label?: string;
    dirty?: boolean;
  } = $props();

  let inputValue = $state("");

  function addItem() {
    const trimmed = inputValue.trim();
    if (!trimmed || items.includes(trimmed)) return;
    items = [...items, trimmed];
    inputValue = "";
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

<div class="flex flex-col gap-2">
  {#if label}
    <label class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)">
      {label}
      <DirtyDot {dirty} />
    </label>
  {/if}

  <ul class="flex flex-col gap-1">
    {#each items as item, i}
      <li class="group flex items-center gap-1">
        <code class="flex-1 rounded px-2 py-1 text-xs font-mono break-all" style="background-color: var(--bg-tertiary); color: var(--text-primary)">{item}</code>
        <button
          onclick={() => removeItem(i)}
          class="btn-danger-ghost opacity-0 group-hover:opacity-100 transition-opacity rounded p-1"
          aria-label="Remove {item}"
        >
          ×
        </button>
      </li>
    {/each}
  </ul>

  <div class="flex gap-2">
    <input
      type="text"
      bind:value={inputValue}
      onkeydown={handleKeydown}
      {placeholder}
      class="input-base flex-1 rounded px-2 py-1 text-sm"
    />
    <button
      onclick={addItem}
      class="btn-primary rounded px-3 py-1 text-sm transition-colors"
    >
      Add
    </button>
  </div>
</div>

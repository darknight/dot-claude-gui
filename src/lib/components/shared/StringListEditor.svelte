<script lang="ts">
  let {
    items = $bindable([]),
    placeholder = "Add item...",
    label,
  }: {
    items: string[];
    placeholder?: string;
    label?: string;
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
    <label class="text-xs font-medium text-gray-400 uppercase tracking-wide">{label}</label>
  {/if}

  <ul class="flex flex-col gap-1">
    {#each items as item, i}
      <li class="group flex items-center gap-1">
        <code class="flex-1 rounded bg-gray-800 px-2 py-1 text-xs text-gray-200 font-mono break-all">{item}</code>
        <button
          onclick={() => removeItem(i)}
          class="opacity-0 group-hover:opacity-100 transition-opacity rounded p-1 text-gray-500 hover:text-red-400 hover:bg-gray-700"
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
      class="flex-1 rounded bg-gray-800 border border-gray-700 px-2 py-1 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-blue-500"
    />
    <button
      onclick={addItem}
      class="rounded bg-blue-600 px-3 py-1 text-sm text-white hover:bg-blue-500 active:bg-blue-700 transition-colors"
    >
      Add
    </button>
  </div>
</div>

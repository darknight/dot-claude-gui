<script lang="ts">
  let { onResize, min = 56, max = 400 }: { onResize: (width: number) => void; min?: number; max?: number } = $props();

  let dragging = $state(false);
  let startX = 0;
  let startWidth = 0;

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    startX = e.clientX;
    // Read the current width from the previous sibling
    const prev = (e.target as HTMLElement).previousElementSibling;
    startWidth = prev ? prev.getBoundingClientRect().width : 0;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    const delta = e.clientX - startX;
    const newWidth = Math.min(max, Math.max(min, startWidth + delta));
    onResize(newWidth);
  }

  function onPointerUp() {
    dragging = false;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="w-1 cursor-col-resize transition-colors hover:bg-[var(--accent-primary)]/50 {dragging ? 'bg-[var(--accent-primary)]/50' : ''}"
  style="flex-shrink: 0"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
></div>

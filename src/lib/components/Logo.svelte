<script lang="ts">
  // A point that echoes outward over time — a memory reinforced by repetition.
  let { size = 28, tile = true }: { size?: number; tile?: boolean } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden="true">
  {#if tile}
    <rect width="32" height="32" rx="8" fill="currentColor" />
    <circle cx="10" cy="16" r="2.5" fill="var(--bg)" />
    <path d="M15 10.5 A 6 6 0 0 1 15 21.5" stroke="var(--bg)" stroke-width="2.1" stroke-linecap="round" />
    <path d="M19 7 A 9.4 9.4 0 0 1 19 25" stroke="var(--bg)" stroke-width="2.1" stroke-linecap="round" opacity="0.5" />
  {:else}
    <circle cx="10" cy="16" r="2.5" fill="currentColor" />
    <path d="M15 10.5 A 6 6 0 0 1 15 21.5" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" />
    <path d="M19 7 A 9.4 9.4 0 0 1 19 25" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" opacity="0.5" />
  {/if}
</svg>

<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import type { Snippet } from "svelte";

  let {
    open = false,
    title = "",
    onclose,
    maxWidth = "520px",
    children,
  }: {
    open?: boolean;
    title?: string;
    onclose?: () => void;
    maxWidth?: string;
    children?: Snippet;
  } = $props();

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose?.();
  }
</script>

<svelte:window {onkeydown} />

{#if open}
  <div class="backdrop" transition:fade={{ duration: 160 }} onclick={() => onclose?.()} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="modal"
      style="max-width:{maxWidth}"
      transition:scale={{ duration: 200, start: 0.97 }}
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <header>
        <h2>{title}</h2>
        <button class="btn-ghost close" onclick={() => onclose?.()} aria-label="Close">
          <Icon name="x" size={18} />
        </button>
      </header>
      <div class="body">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(20, 18, 16, 0.32);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: var(--s20) var(--s4) var(--s8);
    overflow-y: auto;
  }
  .modal {
    width: 100%;
    background: var(--surface);
    border: 1px solid var(--hairline);
    border-radius: var(--r5);
    box-shadow: var(--shadow-3);
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s5) var(--s5) var(--s4);
  }
  h2 {
    font-size: var(--t-md);
    font-weight: 600;
    letter-spacing: var(--tracking-tight);
  }
  .close {
    width: var(--s8);
    height: var(--s8);
    border-radius: var(--r2);
    color: var(--text-2);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .body {
    padding: 0 var(--s5) var(--s5);
  }
  @media (max-width: 640px) {
    .backdrop { padding: var(--s4) 0 0; align-items: flex-end; }
    .modal { border-radius: var(--r5) var(--r5) 0 0; max-width: none !important; }
  }
</style>

<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { Snippet } from "svelte";

  let {
    onclick,
    children,
    variant = "default",
    full = false,
    disabled = false,
  }: {
    onclick: () => Promise<unknown> | unknown;
    children: Snippet;
    variant?: "primary" | "default";
    full?: boolean;
    disabled?: boolean;
  } = $props();

  let phase = $state<"idle" | "loading" | "done" | "error">("idle");
  let timer: ReturnType<typeof setTimeout>;

  async function run() {
    if (phase === "loading" || disabled) return;
    phase = "loading";
    try {
      await onclick();
      phase = "done";
    } catch {
      phase = "error";
    }
    clearTimeout(timer);
    timer = setTimeout(() => (phase = "idle"), 1400);
  }
</script>

<button
  class="btn"
  class:btn-primary={variant === "primary"}
  class:full
  class:busy={phase !== "idle"}
  {disabled}
  onclick={run}
>
  <span class="content" class:hidden={phase !== "idle"}>{@render children()}</span>
  {#if phase === "loading"}
    <span class="overlay"><span class="spinner"></span></span>
  {:else if phase === "done"}
    <span class="overlay"><Icon name="check" size={18} stroke={2.2} /></span>
  {:else if phase === "error"}
    <span class="overlay"><Icon name="x" size={18} stroke={2.2} /></span>
  {/if}
</button>

<style>
  .btn {
    position: relative;
  }
  .full {
    width: 100%;
  }
  .content {
    display: inline-flex;
    align-items: center;
    gap: var(--s2);
    transition: opacity var(--dur1) var(--ease);
  }
  .content.hidden {
    opacity: 0;
  }
  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    animation: pop var(--dur1) var(--ease-out);
  }
  .spinner {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid currentColor;
    border-top-color: transparent;
    opacity: 0.55;
    animation: abspin 0.65s linear infinite;
  }
  @keyframes abspin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

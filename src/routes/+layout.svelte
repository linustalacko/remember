<script lang="ts">
  import "$lib/styles/app.css";
  import { page } from "$app/stores";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import { bumpData } from "$lib/app.svelte";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  // While reviewing a deck, hide all chrome — just the cards.
  let focused = $derived($page.url.pathname.startsWith("/study"));

  // Refetch when the window regains focus (background sync may have brought
  // new data from another device).
  $effect(() => {
    const refresh = () => bumpData();
    const onVis = () => document.visibilityState === "visible" && bumpData();
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", onVis);
    return () => {
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", onVis);
    };
  });
</script>

<!-- Draggable strip across the top (the window has no title bar) -->
<div class="drag-region" data-tauri-drag-region></div>

<div class="app" class:focused>
  {#if !focused}
    <Sidebar />
  {/if}
  <main>
    {@render children()}
  </main>
</div>

<CardEditor />

<style>
  .drag-region {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: var(--titlebar);
    z-index: 30;
    -webkit-app-region: drag;
  }
  .app {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-areas: "nav main";
    height: 100vh;
  }
  .app.focused {
    grid-template-columns: 1fr;
    grid-template-areas: "main";
  }
  main {
    grid-area: main;
    height: 100vh;
    overflow-y: auto;
    overflow-x: hidden;
  }
  @media (max-width: 880px) {
    .drag-region {
      display: none;
    }
    .app {
      grid-template-columns: 1fr;
      grid-template-areas: "main";
    }
    main {
      padding-bottom: calc(var(--s16) + env(safe-area-inset-bottom, 0px));
    }
    .app.focused main {
      padding-bottom: 0;
    }
  }
</style>

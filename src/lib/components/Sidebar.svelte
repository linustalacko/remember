<script lang="ts">
  import { page } from "$app/stores";
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";
  import { openEditor } from "$lib/app.svelte";
  import { api } from "$lib/api";

  const nav = [
    // Study is "home" — active for "/", "/study", or any path that isn't one of
    // the others (robust against the initial route being "/index.html" etc.)
    {
      href: "/",
      icon: "study",
      label: "Study",
      match: (p: string) =>
        !p.startsWith("/browse") && !p.startsWith("/stats") && !p.startsWith("/settings"),
    },
    { href: "/browse", icon: "browse", label: "Browse", match: (p: string) => p.startsWith("/browse") },
    { href: "/stats", icon: "stats", label: "Stats", match: (p: string) => p.startsWith("/stats") },
    { href: "/settings", icon: "settings", label: "Settings", match: (p: string) => p.startsWith("/settings") },
  ];

  let pathname = $derived($page.url.pathname);
  let activeIndex = $derived(nav.findIndex((i) => i.match(pathname)));

  let syncPhase = $state<"idle" | "loading" | "done" | "error">("idle");
  let syncTimer: ReturnType<typeof setTimeout>;

  async function doSync() {
    if (syncPhase === "loading") return;
    syncPhase = "loading";
    try {
      await api.syncNow();
      syncPhase = "done";
    } catch {
      syncPhase = "error";
    }
    clearTimeout(syncTimer);
    syncTimer = setTimeout(() => (syncPhase = "idle"), 1400);
  }

  const syncLabel = $derived(
    syncPhase === "loading"
      ? "Syncing…"
      : syncPhase === "done"
        ? "Synced"
        : syncPhase === "error"
          ? "Failed"
          : "Sync",
  );
  const syncIcon = $derived(
    syncPhase === "done" ? "check" : syncPhase === "error" ? "x" : "sync",
  );
</script>

<!-- Desktop sidebar -->
<nav class="sidebar">
  <div class="brand row">
    <span class="slot"><Logo size={20} /></span>
    <span class="word">Remember</span>
  </div>

  <button class="row add" onclick={() => openEditor()}>
    <span class="slot"><Icon name="plus" size={20} /></span>
    <span>Add card</span>
  </button>

  <ul class="links">
    {#if activeIndex >= 0}
      <span
        class="indicator"
        style="transform: translateY(calc({activeIndex} * (var(--nav-h) + var(--s1))))"
      ></span>
    {/if}
    {#each nav as item, i}
      <li>
        <a href={item.href} class="row nav" class:active={i === activeIndex}>
          <span class="slot"><Icon name={item.icon} size={20} /></span>
          <span>{item.label}</span>
        </a>
      </li>
    {/each}
  </ul>

  <button class="row nav sync" onclick={doSync} disabled={syncPhase === "loading"}>
    <span class="slot" class:spin={syncPhase === "loading"}><Icon name={syncIcon} size={20} /></span>
    <span>{syncLabel}</span>
  </button>
</nav>

<!-- Mobile tab bar -->
<nav class="tabbar">
  <a href="/" class:active={pathname === "/" || pathname.startsWith("/study")}>
    <Icon name="study" size={22} /><span>Study</span>
  </a>
  <a href="/browse" class:active={pathname.startsWith("/browse")}>
    <Icon name="browse" size={22} /><span>Browse</span>
  </a>
  <button class="fab" onclick={() => openEditor()} aria-label="Add card">
    <Icon name="plus" size={24} />
  </button>
  <a href="/stats" class:active={pathname.startsWith("/stats")}>
    <Icon name="stats" size={22} /><span>Stats</span>
  </a>
  <a href="/settings" class:active={pathname.startsWith("/settings")}>
    <Icon name="settings" size={22} /><span>Settings</span>
  </a>
</nav>

<style>
  .sidebar {
    grid-area: nav;
    width: 248px;
    height: 100vh;
    padding: calc(var(--titlebar) + var(--s4)) var(--s3) var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s1);
    border-right: 1px solid var(--hairline);
    background: var(--surface-2);
  }

  /* every sidebar row shares one left column via the icon slot */
  .row {
    display: flex;
    align-items: center;
    gap: var(--s3);
    width: 100%;
    height: var(--nav-h);
    padding: 0 var(--s5);
    border-radius: var(--r3);
    font-size: var(--t-base);
    font-weight: 520;
    text-align: left;
    color: var(--text-2);
  }
  .slot {
    flex: none;
    width: var(--s6);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: inherit;
  }

  .brand {
    color: var(--text);
    margin-bottom: var(--s2);
    cursor: default;
  }
  .word {
    font-size: var(--t-md);
    font-weight: 640;
    letter-spacing: var(--tracking-tight);
  }

  .add {
    background: var(--accent);
    color: var(--accent-text);
    margin-bottom: var(--s3);
    box-shadow: var(--shadow-1);
    transition: background var(--dur1) var(--ease), transform var(--dur1) var(--ease);
  }
  .add:hover { background: var(--accent-hover); }
  .add:active { transform: scale(0.99); }

  .links {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--s1);
    flex: 1;
    position: relative;
  }
  /* the selected pill — slides between items, no shadow */
  .indicator {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: var(--nav-h);
    border-radius: var(--r3);
    background: var(--nav-pill);
    border: 1px solid var(--hairline);
    transition: transform 0.24s cubic-bezier(0.3, 0.72, 0.15, 1);
    z-index: 0;
  }
  .nav {
    position: relative;
    z-index: 1;
    background: transparent;
    transition: color var(--dur1) var(--ease), background var(--dur1) var(--ease);
  }
  .nav:hover:not(.active) {
    background: rgba(255, 255, 255, 0.45);
    color: var(--text);
  }
  .nav.active {
    color: var(--text);
    font-weight: 620;
  }
  .sync:disabled { opacity: 0.6; }
  .spin :global(svg) { animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .tabbar { display: none; }

  @media (prefers-color-scheme: dark) {
    .nav:hover:not(.active) { background: rgba(255, 255, 255, 0.06); }
  }

  @media (max-width: 880px) {
    .sidebar { display: none; }
    .tabbar {
      grid-area: nav;
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      z-index: 50;
      height: calc(var(--s16) + env(safe-area-inset-bottom, 0px));
      padding-bottom: env(safe-area-inset-bottom, 0px);
      display: flex;
      align-items: center;
      justify-content: space-around;
      background: color-mix(in srgb, var(--surface) 88%, transparent);
      backdrop-filter: blur(18px);
      -webkit-backdrop-filter: blur(18px);
      border-top: 1px solid var(--hairline);
    }
    .tabbar a {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
      color: var(--text-3);
      font-size: var(--t-xs);
      font-weight: 540;
      width: var(--s16);
    }
    .tabbar a.active { color: var(--accent); }
    .fab {
      display: flex;
      align-items: center;
      justify-content: center;
      width: var(--s12);
      height: var(--s12);
      margin-top: calc(var(--s6) * -1);
      border-radius: var(--r-full);
      background: var(--accent);
      color: var(--accent-text);
      box-shadow: var(--shadow-2);
    }
    .fab:active { transform: scale(0.94); }
  }
</style>

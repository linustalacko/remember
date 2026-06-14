<script lang="ts">
  import { api, type ImportSummary } from "$lib/api";
  import { bumpData } from "$lib/app.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import ActionButton from "$lib/components/ActionButton.svelte";

  let ankiPath = $state("");
  let summary = $state<ImportSummary | null>(null);

  let newPerDay = $state(20);
  let reviewsPerDay = $state(200);

  let syncUrl = $state("");
  let syncToken = $state("");
  let syncConfigured = $state(false);

  $effect(() => {
    void init();
  });

  async function init() {
    try {
      const [path, settings, sync] = await Promise.all([
        api.defaultAnkiPath(),
        api.getSettings(),
        api.getSyncStatus(),
      ]);
      if (path) ankiPath = path;
      newPerDay = settings.new_per_day;
      reviewsPerDay = settings.reviews_per_day;
      syncConfigured = sync.configured;
      syncUrl = sync.url;
    } catch {
      // surfaced on the relevant button if a save fails
    }
  }

  // Handlers throw on failure so the button shows its error state.
  async function runImport() {
    if (!ankiPath.trim()) throw new Error("missing path");
    summary = await api.importAnki(ankiPath.trim());
    bumpData();
  }

  async function saveLimits() {
    await api.setSetting("new_per_day", String(newPerDay));
    await api.setSetting("reviews_per_day", String(reviewsPerDay));
    bumpData();
  }

  async function saveSync() {
    await api.setSyncConfig(syncUrl.trim(), syncToken.trim());
    syncConfigured = !!syncUrl.trim();
  }

  async function syncNow() {
    await api.syncNow();
  }
</script>

<div class="page">
  <header class="pagehead"><h1>Settings</h1></header>

  <section class="card-surface">
    <div class="shead">
      <h2>Import from Anki</h2>
    </div>
    <div class="field">
      <span class="label">Collection file</span>
      <input class="input" bind:value={ankiPath} placeholder="…/Anki2/User 1/collection.anki2" />
    </div>
    <div class="srow">
      <ActionButton variant="primary" onclick={runImport}>Import</ActionButton>
      {#if summary}
        <span class="summary">
          <Icon name="check" size={16} />
          {summary.decks} decks · {summary.notes} notes · {summary.cards} cards · {summary.reviews} reviews
        </span>
      {/if}
    </div>
  </section>

  <section class="card-surface">
    <div class="shead">
      <h2>Daily limits</h2>
    </div>
    <div class="grid2">
      <div class="field">
        <span class="label">New cards / day</span>
        <input class="input" type="number" min="0" bind:value={newPerDay} />
      </div>
      <div class="field">
        <span class="label">Reviews / day</span>
        <input class="input" type="number" min="0" bind:value={reviewsPerDay} />
      </div>
    </div>
    <div class="srow">
      <ActionButton onclick={saveLimits}>Save limits</ActionButton>
    </div>
  </section>

  <section class="card-surface">
    <div class="shead">
      <h2>Sync <span class="badge" class:on={syncConfigured}>{syncConfigured ? "Connected" : "Off"}</span></h2>
    </div>
    <div class="field">
      <span class="label">Database URL</span>
      <input class="input" bind:value={syncUrl} placeholder="libsql://your-db.turso.io" autocomplete="off" />
    </div>
    <div class="field">
      <span class="label">Auth token</span>
      <input class="input" type="password" bind:value={syncToken} placeholder="paste token" autocomplete="off" />
    </div>
    <div class="srow">
      <ActionButton variant="primary" onclick={saveSync}>Save &amp; connect</ActionButton>
      <ActionButton onclick={syncNow} disabled={!syncConfigured}>
        <Icon name="sync" size={16} /> Sync now
      </ActionButton>
    </div>
  </section>
</div>

<style>
  .page {
    max-width: var(--page-w);
    margin: 0 auto;
    padding: var(--s10) var(--s6) var(--s12);
    display: flex;
    flex-direction: column;
    gap: var(--s5);
    animation: rise var(--dur2) var(--ease-out);
  }
  .pagehead h1 { font-size: var(--t-2xl); font-weight: 640; letter-spacing: var(--tracking-tight); }

  section { padding: var(--s6); display: flex; flex-direction: column; gap: var(--s4); }
  .shead { display: flex; flex-direction: column; gap: var(--s1); }
  h2 { font-size: var(--t-md); font-weight: 600; display: flex; align-items: center; gap: var(--s3); }
  .shead p { font-size: var(--t-sm); }

  .field { display: flex; flex-direction: column; gap: var(--s2); }
  .label { padding-left: var(--s1); }
  .grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s4); }
  .srow { display: flex; align-items: center; gap: var(--s3); flex-wrap: wrap; }

  .summary { display: inline-flex; align-items: center; gap: var(--s2); font-size: var(--t-sm); color: var(--good); }

  .badge {
    font-size: var(--t-xs); font-weight: 600; letter-spacing: var(--tracking-wide); text-transform: uppercase;
    padding: 2px var(--s2); border-radius: var(--r-full);
    background: var(--surface-sunken); color: var(--text-3);
  }
  .badge.on { background: var(--good-soft); color: var(--good); }
  .hint { font-size: var(--t-xs); }

  @media (max-width: 880px) {
    .page { padding: var(--s8) var(--s4) var(--s12); }
    .pagehead h1 { font-size: var(--t-xl); }
    .grid2 { grid-template-columns: 1fr; }
  }
</style>

<script lang="ts">
  import { api, type CardRow, type DeckWithCounts } from "$lib/api";
  import { plainPreview } from "$lib/render";
  import { dueLabel, STATE_LABEL } from "$lib/format";
  import { openEditor, toast, ui } from "$lib/app.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let cards = $state<CardRow[]>([]);
  let decks = $state<DeckWithCounts[]>([]);
  let search = $state("");
  let deckFilter = $state<string>("");
  let loading = $state(true);
  let limit = 300;
  let debounce: ReturnType<typeof setTimeout>;

  $effect(() => {
    void api.listDecks().then((d) => (decks = d));
  });

  $effect(() => {
    // dependencies: search, deckFilter, and external data changes
    search;
    deckFilter;
    ui.dataVersion;
    clearTimeout(debounce);
    debounce = setTimeout(load, 160);
    return () => clearTimeout(debounce);
  });

  async function load() {
    loading = true;
    try {
      cards = await api.listCards(deckFilter || null, search.trim(), limit, 0);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      loading = false;
    }
  }

  async function remove(e: MouseEvent, card: CardRow) {
    e.stopPropagation();
    if (!confirm("Delete this card?")) return;
    try {
      await api.deleteNote(card.note_id);
      cards = cards.filter((c) => c.id !== card.id);
      toast("Deleted", "success");
    } catch (err) {
      toast(String(err), "error");
    }
  }
</script>

<div class="page">
  <header class="pagehead">
    <h1>Browse</h1>
    <span class="faint num">{cards.length}{cards.length >= limit ? "+" : ""} cards</span>
  </header>

  <div class="controls">
    <div class="searchbox">
      <Icon name="search" size={18} />
      <input class="searchinput" placeholder="Search cards…" bind:value={search} />
    </div>
    <select class="input select deckselect" bind:value={deckFilter}>
      <option value="">All decks</option>
      {#each decks as d}
        <option value={d.id}>{d.name}</option>
      {/each}
    </select>
  </div>

  {#if loading && cards.length === 0}
    <div class="spinner"></div>
  {:else if cards.length === 0}
    <div class="empty muted">No cards found.</div>
  {:else}
    <div class="rows">
      {#each cards as card (card.id)}
        {@const p = plainPreview(card)}
        <div
          class="row"
          role="button"
          tabindex="0"
          onclick={() => openEditor({ noteId: card.note_id })}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              openEditor({ noteId: card.note_id });
            }
          }}
        >
          <div class="text">
            <span class="q">{p.q || "—"}</span>
            <span class="a faint">{p.a}</span>
          </div>
          <div class="meta">
            <span class="chip deck">{card.deck_name}</span>
            <span class="state {card.state}">{STATE_LABEL[card.state] ?? card.state}</span>
            <span class="due faint num">{dueLabel(card.due, card.state)}</span>
            <button class="del" aria-label="Delete" onclick={(e) => remove(e, card)}>
              <Icon name="trash" size={16} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: var(--page-w);
    margin: 0 auto;
    padding: var(--s10) var(--s6) var(--s12);
    animation: rise var(--dur2) var(--ease-out);
  }
  .pagehead { display: flex; align-items: baseline; justify-content: space-between; margin-bottom: var(--s6); }
  h1 { font-size: var(--t-2xl); font-weight: 640; letter-spacing: var(--tracking-tight); }

  .controls { display: flex; gap: var(--s3); margin-bottom: var(--s5); }
  .searchbox {
    flex: 1; display: flex; align-items: center; gap: var(--s2);
    padding: 0 var(--s4); height: var(--s10);
    background: var(--surface); border: 1px solid var(--hairline-2); border-radius: var(--r2);
    color: var(--text-3);
    transition: border-color var(--dur1) var(--ease), box-shadow var(--dur1) var(--ease);
  }
  .searchbox:focus-within { border-color: var(--accent); box-shadow: var(--focus); }
  .searchinput { flex: 1; border: none; background: none; color: var(--text); height: 100%; }
  .deckselect { width: auto; min-width: 160px; appearance: none; cursor: pointer; }

  .rows { list-style: none; display: flex; flex-direction: column; }
  .row {
    display: flex;
    align-items: center;
    gap: var(--s4);
    padding: var(--s3) var(--s3);
    border-radius: var(--r2);
    border-bottom: 1px solid var(--hairline);
    cursor: pointer;
    transition: background var(--dur1) var(--ease);
  }
  .row:hover { background: var(--surface); }
  .row:last-child { border-bottom: none; }
  .text { flex: 1; display: flex; flex-direction: column; gap: 1px; min-width: 0; }
  .q { font-weight: 540; font-size: var(--t-base); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .a { font-size: var(--t-sm); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta { display: flex; align-items: center; gap: var(--s3); flex: none; }
  .chip.deck { max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .state {
    font-size: var(--t-xs); font-weight: 600; letter-spacing: var(--tracking-wide); text-transform: uppercase;
    color: var(--text-3);
  }
  .state.review { color: var(--good); }
  .state.learning, .state.relearning { color: var(--hard); }
  .state.new { color: var(--easy); }
  .state.suspended { color: var(--text-3); }
  .due { min-width: var(--s12); text-align: right; font-size: var(--t-sm); }
  .del {
    width: var(--s8); height: var(--s8); display: inline-flex; align-items: center; justify-content: center;
    border-radius: var(--r2); color: var(--text-3); opacity: 0;
    transition: opacity var(--dur1) var(--ease), background var(--dur1) var(--ease), color var(--dur1) var(--ease);
  }
  .row:hover .del { opacity: 1; }
  .del:hover { background: var(--again-soft); color: var(--again); }

  .empty { text-align: center; padding: var(--s16); }
  .spinner {
    width: var(--s8); height: var(--s8); margin: var(--s16) auto;
    border-radius: var(--r-full); border: 2px solid var(--hairline-2); border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 880px) {
    .page { padding: var(--s8) var(--s4) var(--s12); }
    h1 { font-size: var(--t-xl); }
    .a { display: none; }
    .chip.deck { display: none; }
    .meta { gap: var(--s2); }
  }
</style>

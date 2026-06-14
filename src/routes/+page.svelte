<script lang="ts">
  import { api, type DeckWithCounts, type Stats } from "$lib/api";
  import { ui, openEditor, toast } from "$lib/app.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let decks = $state<DeckWithCounts[]>([]);
  let stats = $state<Stats | null>(null);
  let loaded = $state(false);

  let totalDue = $derived(
    decks.reduce((s, d) => s + d.new_count + d.learning_count + d.review_count, 0),
  );

  $effect(() => {
    ui.dataVersion; // re-run when data changes
    void load();
  });

  let retries = 0;
  async function load() {
    try {
      decks = await api.listDecks();
    } catch (e) {
      toast(String(e), "error");
    }
    try {
      stats = await api.stats();
    } catch {
      // stats are non-critical for the deck list
    }
    loaded = true;
    // On a cold start the replica may still be applying its first sync —
    // keep retrying briefly until data arrives.
    if (decks.length === 0 && retries < 4) {
      retries++;
      setTimeout(() => void load(), 500);
    } else {
      retries = 0;
    }
  }

  const greeting = () => {
    const h = new Date().getHours();
    if (h < 5) return "Late night";
    if (h < 12) return "Good morning";
    if (h < 18) return "Good afternoon";
    return "Good evening";
  };
</script>

<div class="page">
  <header class="pagehead">
    <div>
      <h1>{greeting()}</h1>
    </div>
    {#if stats}
      <div class="today">
        <div class="stat">
          <span class="num big">{stats.reviews_today}</span>
          <span class="faint cap">reviewed today</span>
        </div>
        <div class="stat">
          <span class="num big flame"><Icon name="flame" size={18} />{stats.streak_days}</span>
          <span class="faint cap">day streak</span>
        </div>
      </div>
    {/if}
  </header>

  {#if totalDue > 0}
    <a class="hero" href="/study">
      <div class="herotext">
        <span class="cap accent">Ready</span>
        <span class="num herocount">{totalDue}</span>
      </div>
      <span class="herostart">Start review <Icon name="chevron" size={18} /></span>
    </a>
  {/if}

  <section>
    <div class="sectionhead">
      <h2>Decks</h2>
      <button class="btn btn-ghost small" onclick={() => openEditor()}>
        <Icon name="plus" size={16} /> Add card
      </button>
    </div>

    {#if loaded && decks.length === 0}
      <div class="empty card-surface">
        <Icon name="decks" size={32} />
        <h3>No decks yet</h3>
        <div class="emptyactions">
          <a class="btn btn-primary" href="/settings">Import from Anki</a>
          <button class="btn" onclick={() => openEditor()}>Add a card</button>
        </div>
      </div>
    {:else}
      <ul class="decklist">
        {#each decks as d (d.id)}
          {@const due = d.new_count + d.learning_count + d.review_count}
          <a class="deckrow" href={`/study?deck=${d.id}`} class:caughtup={due === 0}>
            <span class="deckicon"><Icon name="folder" size={20} /></span>
            <div class="deckmain">
              <span class="deckname">{d.name}</span>
            </div>
            <div class="counts">
              <span class="count new" class:zero={d.new_count === 0}>{d.new_count}</span>
              <span class="count learn" class:zero={d.learning_count === 0}>{d.learning_count}</span>
              <span class="count review" class:zero={d.review_count === 0}>{d.review_count}</span>
            </div>
            <span class="go"><Icon name="chevron" size={18} /></span>
          </a>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .page {
    max-width: var(--page-w);
    margin: 0 auto;
    padding: var(--s10) var(--s6) var(--s12);
    display: flex;
    flex-direction: column;
    gap: var(--s6);
    animation: rise var(--dur2) var(--ease-out);
  }
  .pagehead {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s4);
  }
  h1 {
    font-size: var(--t-2xl);
    font-weight: 640;
    letter-spacing: var(--tracking-tight);
    line-height: var(--lh-tight);
  }
  .sub { margin-top: var(--s2); font-size: var(--t-md); }
  .today { display: flex; gap: var(--s8); }
  .stat { display: flex; flex-direction: column; align-items: flex-end; gap: var(--s1); }
  .num.big { font-size: var(--t-xl); font-weight: 620; }
  .flame { display: inline-flex; align-items: center; gap: var(--s1); color: var(--accent); }
  .cap { font-size: var(--t-xs); letter-spacing: var(--tracking-wide); text-transform: uppercase; }

  .hero {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--s6) var(--s7);
    border-radius: var(--r5);
    background: var(--surface);
    border: 1px solid var(--hairline);
    box-shadow: var(--shadow-2);
    transition: transform var(--dur2) var(--ease-out), box-shadow var(--dur2) var(--ease);
  }
  .hero:hover { transform: translateY(-2px); box-shadow: var(--shadow-3); }
  .herotext { display: flex; flex-direction: column; gap: var(--s1); }
  .accent { color: var(--accent); font-weight: 600; }
  .herocount { font-size: var(--t-4xl); font-weight: 680; line-height: 1; letter-spacing: var(--tracking-tight); }
  .herostart {
    display: inline-flex; align-items: center; gap: var(--s1);
    padding: var(--s3) var(--s5);
    background: var(--accent); color: var(--accent-text);
    border-radius: var(--r-full); font-weight: 560;
  }

  .sectionhead { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--s3); }
  h2 { font-size: var(--t-md); font-weight: 600; }
  .small { font-size: var(--t-sm); }
  .btn.small { height: var(--s8); padding: 0 var(--s3); font-size: var(--t-sm); }

  .decklist { list-style: none; display: flex; flex-direction: column; gap: var(--s2); }
  .deckrow {
    display: flex;
    align-items: center;
    gap: var(--s4);
    padding: var(--s4) var(--s5);
    border-radius: var(--r4);
    background: var(--surface);
    border: 1px solid var(--hairline);
    transition: transform var(--dur1) var(--ease), border-color var(--dur1) var(--ease), box-shadow var(--dur1) var(--ease);
  }
  .deckrow:hover { border-color: var(--hairline-2); box-shadow: var(--shadow-1); transform: translateY(-1px); }
  .deckicon { color: var(--text-3); display: inline-flex; }
  .deckmain { flex: 1; display: flex; flex-direction: column; gap: 1px; min-width: 0; }
  .deckname { font-weight: 560; font-size: var(--t-md); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .counts { display: flex; align-items: center; gap: var(--s2); font-variant-numeric: tabular-nums; }
  .count {
    min-width: var(--s7);
    text-align: center;
    padding: 2px var(--s2);
    border-radius: var(--r-full);
    font-size: var(--t-sm);
    font-weight: 600;
  }
  .count.new { color: var(--easy); background: var(--easy-soft); }
  .count.learn { color: var(--hard); background: var(--hard-soft); }
  .count.review { color: var(--good); background: var(--good-soft); }
  .count.zero { color: var(--text-3); background: transparent; }
  .caughtup .counts { opacity: 0.5; }
  .go { color: var(--text-3); display: inline-flex; }

  .empty {
    display: flex; flex-direction: column; align-items: center; gap: var(--s3);
    padding: var(--s14) var(--s8); text-align: center; color: var(--text-3);
  }
  .empty h3 { font-size: var(--t-lg); font-weight: 600; color: var(--text); }
  .emptyactions { display: flex; gap: var(--s3); margin-top: var(--s3); }

  @media (max-width: 880px) {
    .page { padding: var(--s8) var(--s4) var(--s12); gap: var(--s6); }
    h1 { font-size: var(--t-xl); }
    .today { gap: var(--s5); }
  }
</style>

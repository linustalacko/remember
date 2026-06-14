<script lang="ts">
  import { page } from "$app/stores";
  import { fade } from "svelte/transition";
  import { api, type StudyCard, type StudyCounts } from "$lib/api";
  import { renderCard } from "$lib/render";
  import { openEditor, toast, ui } from "$lib/app.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let deckId = $derived($page.url.searchParams.get("deck"));

  let card = $state<StudyCard | null>(null);
  let counts = $state<StudyCounts>({ new_count: 0, learning_count: 0, review_count: 0 });
  let revealed = $state(false);
  let loading = $state(true);
  let title = $state("Review");
  let reviewed = $state(0);
  let shownAt = 0;

  let rendered = $derived(card ? renderCard(card) : null);
  let done = $derived(!loading && !card);

  $effect(() => {
    deckId;
    void start();
  });

  // Refresh the current card if it was just edited.
  $effect(() => {
    ui.dataVersion;
  });

  async function start() {
    loading = true;
    reviewed = 0;
    try {
      if (deckId) {
        const decks = await api.listDecks();
        title = decks.find((d) => d.id === deckId)?.name ?? "Review";
      } else {
        title = "All decks";
      }
      await refresh();
    } catch (e) {
      toast(String(e), "error");
      loading = false;
    }
  }

  async function refresh() {
    counts = await api.studyCounts(deckId);
    card = await api.nextCard(deckId);
    revealed = false;
    shownAt = Date.now();
    loading = false;
  }

  function reveal() {
    if (card && !revealed) revealed = true;
  }

  async function grade(rating: number) {
    if (!card || !revealed) return;
    const dur = Date.now() - shownAt;
    const id = card.id;
    try {
      await api.answerCard(id, rating, dur);
      reviewed++;
      await refresh();
    } catch (e) {
      toast(String(e), "error");
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if (ui.editor.open) return;
    if (done) return;
    if (!revealed) {
      if (e.key === " " || e.key === "Enter") {
        e.preventDefault();
        reveal();
      }
      return;
    }
    if (e.key >= "1" && e.key <= "4") {
      e.preventDefault();
      void grade(Number(e.key));
    } else if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      void grade(3);
    }
  }

  const grades = [
    { rating: 1, label: "Again", cls: "again", key: "1" },
    { rating: 2, label: "Hard", cls: "hard", key: "2" },
    { rating: 3, label: "Good", cls: "good", key: "3" },
    { rating: 4, label: "Easy", cls: "easy", key: "4" },
  ];
</script>

<svelte:window {onkeydown} />

<div class="study">
  <header>
    <a class="iconbtn" href="/" aria-label="Back"><Icon name="arrowLeft" size={20} /></a>
    <div class="hcenter">
      <span class="htitle">{title}</span>
      <div class="hcounts">
        <span class="count new" class:zero={counts.new_count === 0}>{counts.new_count}</span>
        <span class="count learn" class:zero={counts.learning_count === 0}>{counts.learning_count}</span>
        <span class="count review" class:zero={counts.review_count === 0}>{counts.review_count}</span>
      </div>
    </div>
    {#if card}
      <button class="iconbtn" aria-label="Edit" onclick={() => openEditor({ noteId: card!.note_id })}>
        <Icon name="edit" size={19} />
      </button>
    {:else}
      <span class="iconbtn ghost"></span>
    {/if}
  </header>

  <div class="stage">
    {#if loading}
      <div class="spinner"></div>
    {:else if done}
      <div class="finished" in:fade={{ duration: 240 }}>
        <span class="checkring"><Icon name="check" size={30} /></span>
        <h2>{reviewed > 0 ? "Session complete" : "All caught up"}</h2>
        <p class="muted">
          {#if reviewed > 0}
            You reviewed {reviewed} {reviewed === 1 ? "card" : "cards"}.
          {:else}
            Nothing due here right now.
          {/if}
        </p>
        <div class="finishactions">
          <a class="btn btn-primary" href="/">Back to decks</a>
          <button class="btn" onclick={() => openEditor({ deckId })}>Add a card</button>
        </div>
      </div>
    {:else if card && rendered}
      {#key card.id}
        <article class="card" in:fade={{ duration: 140 }}>
          <div class="cardbody front">{@html rendered.front}</div>
          {#if revealed}
            <hr class="divide" />
            <div class="cardbody back">{@html rendered.back}</div>
            {#if rendered.extra}
              <div class="cardbody extra">{@html rendered.extra}</div>
            {/if}
          {/if}
        </article>
      {/key}
    {/if}
  </div>

  {#if !loading && !done}
    <footer class="dock">
      {#if !revealed}
        <button class="reveal" onclick={reveal}>
          Show answer
        </button>
      {:else}
        <div class="grades">
          {#each grades as g}
            <button class="grade {g.cls}" onclick={() => grade(g.rating)}>
              <span class="glabel">{g.label}</span>
              <span class="gival num">{card?.previews[g.rating - 1] ?? ""}</span>
            </button>
          {/each}
        </div>
      {/if}
    </footer>
  {/if}
</div>

<style>
  .study {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s4);
    padding: calc(var(--titlebar) + var(--s2)) var(--s5) var(--s4);
    border-bottom: 1px solid var(--hairline);
  }
  .iconbtn {
    width: var(--s10);
    height: var(--s10);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--r2);
    color: var(--text-2);
    transition: background var(--dur1) var(--ease), color var(--dur1) var(--ease);
  }
  .iconbtn:hover { background: var(--surface-sunken); color: var(--text); }
  .iconbtn.ghost { pointer-events: none; }
  .hcenter { display: flex; flex-direction: column; align-items: center; gap: var(--s1); }
  .htitle { font-weight: 560; font-size: var(--t-base); }
  .hcounts { display: flex; gap: var(--s2); font-variant-numeric: tabular-nums; }
  .count { min-width: var(--s6); text-align: center; padding: 1px var(--s2); border-radius: var(--r-full); font-size: var(--t-xs); font-weight: 660; }
  .count.new { color: var(--easy); background: var(--easy-soft); }
  .count.learn { color: var(--hard); background: var(--hard-soft); }
  .count.review { color: var(--good); background: var(--good-soft); }
  .count.zero { color: var(--text-3); background: transparent; }

  .stage {
    flex: 1;
    overflow-y: auto;
    display: flex;
    justify-content: center;
    padding: var(--s10) var(--s6);
  }
  .card {
    width: 100%;
    max-width: 640px;
    margin: auto;
    text-align: center;
  }
  .cardbody {
    font-size: var(--t-xl);
    line-height: var(--lh-snug);
    letter-spacing: var(--tracking-tight);
    word-wrap: break-word;
  }
  .cardbody.front { font-weight: 560; }
  .cardbody.back { font-weight: 420; }
  .cardbody :global(i) { color: var(--text-2); font-style: italic; }
  .cardbody :global(small) { display: inline-block; margin-top: var(--s2); font-size: var(--t-base); color: var(--text-2); }
  .cardbody :global(.cloze) { color: var(--accent); font-weight: 600; }
  .divide { border: none; border-top: 1px solid var(--hairline); margin: var(--s7) auto; width: 56px; }
  .extra { margin-top: var(--s5); font-size: var(--t-base); color: var(--text-2); }

  .dock {
    padding: var(--s5) var(--s6) var(--s8);
    border-top: 1px solid var(--hairline);
    background: var(--surface-2);
  }
  .reveal {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s3);
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
    height: var(--s14);
    border-radius: var(--r3);
    background: var(--text);
    color: var(--bg);
    font-size: var(--t-md);
    font-weight: 560;
    transition: transform var(--dur1) var(--ease), opacity var(--dur1) var(--ease);
  }
  .reveal:hover { opacity: 0.9; }
  .reveal:active { transform: scale(0.99); }

  .grades {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--s3);
    max-width: 640px;
    margin: 0 auto;
  }
  .grade {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s1);
    padding: var(--s3) var(--s2) var(--s2);
    height: var(--s16);
    justify-content: center;
    border-radius: var(--r3);
    background: var(--surface);
    border: 1px solid var(--hairline-2);
    transition: transform var(--dur1) var(--ease), border-color var(--dur1) var(--ease), background var(--dur1) var(--ease);
  }
  .grade:hover { transform: translateY(-2px); }
  .grade:active { transform: translateY(0) scale(0.98); }
  .glabel { font-weight: 580; font-size: var(--t-base); }
  .gival { font-size: var(--t-sm); color: var(--text-2); }
  .grade.again { color: var(--again); border-color: var(--again-soft); }
  .grade.again:hover { background: var(--again-soft); border-color: var(--again); }
  .grade.hard { color: var(--hard); border-color: var(--hard-soft); }
  .grade.hard:hover { background: var(--hard-soft); border-color: var(--hard); }
  .grade.good { color: var(--good); border-color: var(--good-soft); }
  .grade.good:hover { background: var(--good-soft); border-color: var(--good); }
  .grade.easy { color: var(--easy); border-color: var(--easy-soft); }
  .grade.easy:hover { background: var(--easy-soft); border-color: var(--easy); }
  .grade .glabel, .grade .gival { color: inherit; }
  .gival { opacity: 0.85; }

  kbd {
    font-family: var(--font);
    font-size: 10px;
    font-weight: 600;
    color: var(--text-3);
    background: var(--surface-sunken);
    border-radius: var(--r1);
    padding: 1px var(--s1);
    min-width: var(--s4);
    text-align: center;
  }
  .reveal kbd { color: var(--bg); background: rgba(255, 255, 255, 0.18); }

  .spinner {
    width: var(--s8); height: var(--s8); margin: auto;
    border-radius: var(--r-full);
    border: 2px solid var(--hairline-2);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .finished {
    margin: auto;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s3);
  }
  .checkring {
    width: var(--s16); height: var(--s16);
    display: inline-flex; align-items: center; justify-content: center;
    border-radius: var(--r-full);
    background: var(--good-soft); color: var(--good);
    margin-bottom: var(--s2);
  }
  .finished h2 { font-size: var(--t-xl); font-weight: 620; letter-spacing: var(--tracking-tight); }
  .finishactions { display: flex; gap: var(--s3); margin-top: var(--s4); }

  @media (max-width: 880px) {
    .stage { padding: var(--s7) var(--s4); }
    .cardbody { font-size: var(--t-lg); }
    .dock { padding: var(--s4) var(--s4) var(--s6); }
    .grades { gap: var(--s2); }
    .grade { height: var(--s14); }
    .gival { font-size: var(--t-xs); }
  }
</style>

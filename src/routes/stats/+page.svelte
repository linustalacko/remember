<script lang="ts">
  import { api, type Stats } from "$lib/api";
  import { ui, toast } from "$lib/app.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let s = $state<Stats | null>(null);

  $effect(() => {
    ui.dataVersion;
    void api
      .stats()
      .then((r) => (s = r))
      .catch((e) => toast(String(e), "error"));
  });

  let maxCount = $derived(s ? Math.max(1, ...s.history.map((h) => h.count)) : 1);
  let totalReviews = $derived(s ? s.history.reduce((a, h) => a + h.count, 0) : 0);

  function barLabel(day: string, i: number): string {
    if (i % 5 !== 0) return "";
    const d = new Date(day + "T00:00:00");
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }
</script>

<div class="page">
  <header class="pagehead"><h1>Statistics</h1></header>

  {#if s}
    <div class="metrics">
      <div class="metric">
        <span class="mval num"><Icon name="flame" size={22} />{s.streak_days}</span>
        <span class="mlabel">Day streak</span>
      </div>
      <div class="metric">
        <span class="mval num">{s.reviews_today}</span>
        <span class="mlabel">Reviewed today</span>
      </div>
      <div class="metric">
        <span class="mval num">{s.due_today}</span>
        <span class="mlabel">Due today</span>
      </div>
      <div class="metric">
        <span class="mval num">{s.mature_cards}</span>
        <span class="mlabel">Mature cards</span>
      </div>
    </div>

    <section class="card-surface chart">
      <div class="chead">
        <h2>Reviews</h2>
        <span class="faint num">{totalReviews} in 30 days</span>
      </div>
      <div class="bars">
        {#each s.history as h, i}
          <div class="barcol" title={`${h.day}: ${h.count}`}>
            <div class="bar" style="height:{(h.count / maxCount) * 100}%" class:empty={h.count === 0}></div>
            <span class="blabel">{barLabel(h.day, i)}</span>
          </div>
        {/each}
      </div>
    </section>

    <section class="card-surface breakdown">
      <div class="chead"><h2>Collection</h2><span class="faint num">{s.total_cards} cards</span></div>
      <div class="stack">
        {#if s.total_cards > 0}
          <div class="stackbar">
            <span class="seg new" style="flex:{s.new_cards || 0}"></span>
            <span class="seg learn" style="flex:{s.learning_cards || 0}"></span>
            <span class="seg review" style="flex:{s.review_cards || 0}"></span>
            <span class="seg susp" style="flex:{s.suspended_cards || 0}"></span>
          </div>
        {/if}
        <ul class="legend">
          <li><span class="dot new"></span>New<span class="lv num">{s.new_cards}</span></li>
          <li><span class="dot learn"></span>Learning<span class="lv num">{s.learning_cards}</span></li>
          <li><span class="dot review"></span>Review<span class="lv num">{s.review_cards}</span></li>
          <li><span class="dot susp"></span>Suspended<span class="lv num">{s.suspended_cards}</span></li>
        </ul>
      </div>
    </section>
  {:else}
    <div class="spinner"></div>
  {/if}
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
  .pagehead h1 { font-size: var(--t-2xl); font-weight: 640; letter-spacing: var(--tracking-tight); }

  .metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--s3); }
  .metric {
    display: flex; flex-direction: column; gap: var(--s1);
    padding: var(--s5); border-radius: var(--r4);
    background: var(--surface); border: 1px solid var(--hairline);
  }
  .mval { font-size: var(--t-2xl); font-weight: 640; line-height: 1; display: inline-flex; align-items: center; gap: var(--s1); }
  .metric:first-child .mval { color: var(--accent); }
  .mlabel { font-size: var(--t-sm); color: var(--text-2); }

  .chart, .breakdown { padding: var(--s6); }
  .chead { display: flex; align-items: baseline; justify-content: space-between; margin-bottom: var(--s5); }
  h2 { font-size: var(--t-md); font-weight: 600; }

  .bars { display: flex; align-items: flex-end; gap: 3px; height: 168px; }
  .barcol { flex: 1; height: 100%; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; gap: var(--s2); position: relative; }
  .bar {
    width: 100%;
    min-height: 2px;
    background: var(--accent);
    border-radius: var(--r1) var(--r1) 2px 2px;
    transition: opacity var(--dur1) var(--ease);
  }
  .bar.empty { background: var(--hairline-2); }
  .barcol:hover .bar { opacity: 0.7; }
  .blabel { position: absolute; bottom: -20px; font-size: 10px; color: var(--text-3); white-space: nowrap; }

  .stackbar { display: flex; height: var(--s3); border-radius: var(--r-full); overflow: hidden; margin-bottom: var(--s5); background: var(--surface-sunken); }
  .seg.new { background: var(--easy); }
  .seg.learn { background: var(--hard); }
  .seg.review { background: var(--good); }
  .seg.susp { background: var(--text-3); }
  .legend { list-style: none; display: grid; grid-template-columns: 1fr 1fr; gap: var(--s3) var(--s8); }
  .legend li { display: flex; align-items: center; gap: var(--s2); font-size: var(--t-base); }
  .dot { width: var(--s2); height: var(--s2); border-radius: var(--r-full); }
  .dot.new { background: var(--easy); }
  .dot.learn { background: var(--hard); }
  .dot.review { background: var(--good); }
  .dot.susp { background: var(--text-3); }
  .lv { margin-left: auto; font-weight: 600; }

  .spinner { width: var(--s8); height: var(--s8); margin: var(--s16) auto; border-radius: var(--r-full); border: 2px solid var(--hairline-2); border-top-color: var(--accent); animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 880px) {
    .page { padding: var(--s8) var(--s4) var(--s12); }
    .pagehead h1 { font-size: var(--t-xl); }
    .metrics { grid-template-columns: repeat(2, 1fr); }
    .legend { grid-template-columns: 1fr; }
  }
</style>

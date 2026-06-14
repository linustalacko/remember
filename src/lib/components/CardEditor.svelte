<script lang="ts">
  import Modal from "./Modal.svelte";
  import Icon from "./Icon.svelte";
  import ActionButton from "./ActionButton.svelte";
  import { ui, closeEditor, bumpData } from "$lib/app.svelte";
  import { api, NOTE_TYPES, type DeckWithCounts, type Fields } from "$lib/api";

  let decks = $state<DeckWithCounts[]>([]);
  let deckId = $state<string | null>(null);
  let lastDeckId = $state<string | null>(null);
  let noteType = $state("basic");
  let fieldNames = $state<string[]>(NOTE_TYPES.basic.fields);
  let values = $state<Record<string, string>>({});
  let tags = $state("");
  let editing = $state(false);
  let lastOpen = false;
  let creatingDeck = $state(false);
  let newDeckName = $state("");
  let firstField: HTMLTextAreaElement | null = null;

  function autofocus(node: HTMLTextAreaElement, active: boolean) {
    if (active) {
      firstField = node;
      setTimeout(() => node.focus(), 60);
    }
    return {
      destroy() {
        if (firstField === node) firstField = null;
      },
    };
  }

  $effect(() => {
    if (ui.editor.open && !lastOpen) {
      lastOpen = true;
      void init();
    } else if (!ui.editor.open) {
      lastOpen = false;
    }
  });

  async function init() {
    decks = await api.listDecks();
    editing = !!ui.editor.noteId;
    creatingDeck = false;
    tags = "";
    if (editing) {
      const note = await api.getNote(ui.editor.noteId!);
      noteType = note.note_type;
      fieldNames = note.fields.map((f) => f.name);
      values = Object.fromEntries(note.fields.map((f) => [f.name, f.value]));
      tags = note.tags;
    } else {
      noteType = "basic";
      applyType();
      deckId = ui.editor.deckId ?? lastDeckId ?? decks[0]?.id ?? null;
    }
    setTimeout(() => firstField?.focus(), 60);
  }

  function applyType() {
    fieldNames = NOTE_TYPES[noteType]?.fields ?? ["Front", "Back"];
    const next: Record<string, string> = {};
    for (const n of fieldNames) next[n] = values[n] ?? "";
    values = next;
  }

  function onTypeChange() {
    if (!editing) applyType();
  }

  async function makeDeck() {
    const name = newDeckName.trim();
    if (!name) return;
    const id = await api.createDeck(name);
    decks = await api.listDecks();
    deckId = id;
    creatingDeck = false;
    newDeckName = "";
    bumpData();
  }

  // Throws on failure so the save button shows its error state.
  async function save(again = false) {
    const fields: Fields = fieldNames.map((n) => ({ name: n, value: (values[n] ?? "").trim() }));
    if (fields.every((f) => !f.value)) throw new Error("empty");
    if (!editing && !deckId) throw new Error("no deck");
    if (editing) {
      await api.updateNote(ui.editor.noteId!, fields, tags.trim());
    } else {
      await api.createNote(deckId!, noteType, fields, tags.trim());
      lastDeckId = deckId;
    }
    bumpData();
    if (again && !editing) {
      for (const n of fieldNames) values[n] = "";
      setTimeout(() => firstField?.focus(), 30);
    } else {
      closeEditor();
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      save(e.shiftKey).catch(() => {});
    }
  }
</script>

<Modal open={ui.editor.open} title={editing ? "Edit card" : "New card"} onclose={closeEditor} maxWidth="540px">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="form" {onkeydown}>
    {#if !editing}
      <div class="row two">
        <div class="field">
          <span class="label">Deck</span>
          {#if creatingDeck}
            <div class="inline">
              <input class="input" placeholder="Deck name" bind:value={newDeckName}
                onkeydown={(e) => e.key === "Enter" && makeDeck()} />
              <button class="btn" onclick={makeDeck}>Create</button>
            </div>
          {:else}
            <div class="inline">
              <select class="input select" bind:value={deckId}>
                {#each decks as d}
                  <option value={d.id}>{d.name}</option>
                {/each}
              </select>
              <button class="btn icon-btn" title="New deck" onclick={() => (creatingDeck = true)}>
                <Icon name="plus" size={18} />
              </button>
            </div>
          {/if}
        </div>
        <div class="field">
          <span class="label">Type</span>
          <select class="input select" bind:value={noteType} onchange={onTypeChange}>
            {#each Object.entries(NOTE_TYPES) as [key, t]}
              <option value={key}>{t.label}</option>
            {/each}
          </select>
        </div>
      </div>
    {/if}

    {#each fieldNames as name, i}
      <div class="field">
        <span class="label">{name}</span>
        <textarea
          class="input"
          rows={i === 0 ? 2 : 3}
          bind:value={values[name]}
          use:autofocus={i === 0}
        ></textarea>
      </div>
    {/each}

    <div class="field">
      <span class="label">Tags</span>
      <input class="input" placeholder="space separated" bind:value={tags} />
    </div>

    <div class="actions">
      <div class="spacer"></div>
      {#if !editing}
        <ActionButton onclick={() => save(true)}>Save &amp; add</ActionButton>
      {/if}
      <ActionButton variant="primary" onclick={() => save(false)}>
        {editing ? "Save" : "Add card"}
      </ActionButton>
    </div>
  </div>
</Modal>

<style>
  .form { display: flex; flex-direction: column; gap: var(--s4); }
  .row.two { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s4); }
  .field { display: flex; flex-direction: column; gap: var(--s2); }
  .label { padding-left: var(--s1); }
  .select { appearance: none; cursor: pointer; }
  .inline { display: flex; gap: var(--s2); }
  .inline .input { flex: 1; }
  .icon-btn { width: var(--s10); padding: 0; flex: none; }
  textarea.input { font-size: var(--t-md); line-height: var(--lh-snug); }
  .actions { display: flex; align-items: center; gap: var(--s3); margin-top: var(--s1); }
  .spacer { flex: 1; }
  .hint { font-size: var(--t-xs); }
  @media (max-width: 640px) {
    .row.two { grid-template-columns: 1fr; }
    .hint { display: none; }
  }
</style>

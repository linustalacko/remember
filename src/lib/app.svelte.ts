// Shared, app-wide reactive UI state (Svelte 5 runes module).

export const ui = $state({
  editor: {
    open: false,
    noteId: null as string | null,
    deckId: null as string | null,
  },
  /** Bump to ask deck-dependent views to refetch. */
  dataVersion: 0,
});

// No toasts in the app — feedback is shown directly on the buttons. This stays
// as a no-op so existing call sites compile, but nothing is ever displayed.
export function toast(_msg: string, _kind: "info" | "success" | "error" = "info") {}

export function openEditor(opts: { noteId?: string | null; deckId?: string | null } = {}) {
  ui.editor = {
    open: true,
    noteId: opts.noteId ?? null,
    deckId: opts.deckId ?? null,
  };
}

export function closeEditor() {
  ui.editor.open = false;
}

export function bumpData() {
  ui.dataVersion++;
}

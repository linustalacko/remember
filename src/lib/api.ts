import { invoke } from "@tauri-apps/api/core";

export interface Field {
  name: string;
  value: string;
}
export type Fields = Field[];

export interface DeckWithCounts {
  id: string;
  name: string;
  new_count: number;
  learning_count: number;
  review_count: number;
  total: number;
}

export interface StudyCounts {
  new_count: number;
  learning_count: number;
  review_count: number;
}

export interface StudyCard {
  id: string;
  note_id: string;
  deck_id: string;
  deck_name: string;
  note_type: string;
  template: number;
  fields: Fields;
  tags: string;
  state: string;
  interval: number;
  ease: number;
  reps: number;
  lapses: number;
  due: number;
  previews: [string, string, string, string];
}

export interface CardRow {
  id: string;
  note_id: string;
  deck_id: string;
  deck_name: string;
  note_type: string;
  template: number;
  fields: Fields;
  tags: string;
  state: string;
  due: number;
  interval: number;
  reps: number;
  lapses: number;
}

export interface DayCount {
  day: string;
  count: number;
}

export interface Stats {
  total_cards: number;
  new_cards: number;
  learning_cards: number;
  review_cards: number;
  suspended_cards: number;
  reviews_today: number;
  due_today: number;
  due_tomorrow: number;
  streak_days: number;
  mature_cards: number;
  history: DayCount[];
}

export interface ImportSummary {
  decks: number;
  notes: number;
  cards: number;
  reviews: number;
}

export interface NoteData {
  id: string;
  note_type: string;
  fields: Fields;
  tags: string;
}

export const api = {
  listDecks: () => invoke<DeckWithCounts[]>("list_decks"),
  createDeck: (name: string) => invoke<string>("create_deck", { name }),
  renameDeck: (id: string, name: string) => invoke<void>("rename_deck", { id, name }),
  deleteDeck: (id: string) => invoke<void>("delete_deck", { id }),

  studyCounts: (deckId: string | null) => invoke<StudyCounts>("study_counts", { deckId }),
  nextCard: (deckId: string | null) => invoke<StudyCard | null>("next_card", { deckId }),
  answerCard: (cardId: string, rating: number, durationMs: number) =>
    invoke<void>("answer_card", { cardId, rating, durationMs }),

  createNote: (deckId: string, noteType: string, fields: Fields, tags: string) =>
    invoke<string>("create_note", { deckId, noteType, fields, tags }),
  getNote: (noteId: string) => invoke<NoteData>("get_note", { noteId }),
  updateNote: (noteId: string, fields: Fields, tags: string) =>
    invoke<void>("update_note", { noteId, fields, tags }),
  deleteNote: (noteId: string) => invoke<void>("delete_note", { noteId }),
  moveCard: (cardId: string, deckId: string) => invoke<void>("move_card", { cardId, deckId }),

  listCards: (deckId: string | null, search: string, limit: number, offset: number) =>
    invoke<CardRow[]>("list_cards", { deckId, search, limit, offset }),

  stats: () => invoke<Stats>("stats"),
  getSettings: () => invoke<{ new_per_day: number; reviews_per_day: number }>("get_settings"),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),

  getSyncStatus: () => invoke<{ configured: boolean; url: string }>("get_sync_status"),
  setSyncConfig: (url: string, token: string) => invoke<void>("set_sync_config", { url, token }),
  syncNow: () => invoke<string>("sync_now"),

  defaultAnkiPath: () => invoke<string | null>("default_anki_path"),
  importAnki: (path: string) => invoke<ImportSummary>("import_anki", { path }),
};

/** Field templates for each note type when creating a new note. */
export const NOTE_TYPES: Record<string, { label: string; fields: string[] }> = {
  basic: { label: "Basic", fields: ["Front", "Back"] },
  basic_reversed: { label: "Basic + reverse", fields: ["Front", "Back"] },
  vocab: { label: "Vocabulary", fields: ["Word", "Definition"] },
  cloze: { label: "Cloze", fields: ["Text", "Back Extra"] },
};

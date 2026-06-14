import DOMPurify from "dompurify";
import type { Fields, StudyCard, CardRow } from "./api";

export function fieldVal(fields: Fields, name: string): string {
  const f = fields.find((x) => x.name.toLowerCase() === name.toLowerCase());
  return f ? f.value : "";
}

export interface Rendered {
  front: string;
  back: string;
  extra: string;
}

// Card/note fields are arbitrary user-supplied HTML — typed by the user or, more
// dangerously, imported verbatim from a third-party Anki deck. They're rendered
// with {@html}, so we sanitize here (the single choke point) to strip <script>,
// event handlers, and other injection vectors before they ever reach the DOM.
// Defense-in-depth alongside the app's Content-Security-Policy (tauri.conf.json).
const SANITIZE = {
  FORBID_TAGS: ["script", "style", "iframe", "object", "embed", "form", "base", "meta", "link"],
};

function clean(html: string): string {
  // No DOM during SSR/prerender; sanitization runs in the webview where it matters.
  if (typeof window === "undefined") return html;
  return DOMPurify.sanitize(html, SANITIZE);
}

function cleanRendered(r: Rendered): Rendered {
  return { front: clean(r.front), back: clean(r.back), extra: clean(r.extra) };
}

/** Turn a card's note fields into front / back / extra HTML, honouring its
 *  note type and which template (front-side vs reverse) it represents.
 *  All returned HTML is sanitized. */
export function renderCard(card: StudyCard | CardRow): Rendered {
  const f = card.fields ?? [];
  const reverse = card.template === 1;

  if (card.note_type === "vocab") {
    return cleanRendered({ front: fieldVal(f, "Word"), back: fieldVal(f, "Definition"), extra: "" });
  }
  if (card.note_type === "cloze") {
    return cleanRendered(renderCloze(fieldVal(f, "Text"), fieldVal(f, "Back Extra")));
  }

  let front = fieldVal(f, "Front");
  let back = fieldVal(f, "Back");
  if (!front && f.length > 0) front = f[0].value;
  if (!back && f.length > 1) back = f.slice(1).map((x) => x.value).filter(Boolean).join("<br>");

  const extra = [fieldVal(f, "Extra"), fieldVal(f, "Source")].filter(Boolean).join("<br>");
  if (reverse) return cleanRendered({ front: back, back: front, extra });
  return cleanRendered({ front, back, extra });
}

function renderCloze(text: string, extra: string): Rendered {
  const front = text.replace(/\{\{c\d+::(.*?)(::.*?)?\}\}/g, "<span class='cloze'>[…]</span>");
  const back = text.replace(/\{\{c\d+::(.*?)(::.*?)?\}\}/g, "<span class='cloze'>$1</span>");
  return { front, back, extra };
}

/** A short plain-text preview of a card for the browse table. */
export function plainPreview(card: CardRow): { q: string; a: string } {
  const r = renderCard(card);
  return { q: strip(r.front), a: strip(r.back) };
}

function strip(html: string): string {
  return html
    .replace(/<[^>]*>/g, " ")
    .replace(/&mdash;/g, "—")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/\s+/g, " ")
    .trim();
}

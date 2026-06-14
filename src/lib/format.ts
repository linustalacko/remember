const DAY = 86_400_000;

/** Human label for a due timestamp relative to now. */
export function dueLabel(due: number, state: string): string {
  if (state === "new") return "New";
  if (state === "suspended") return "Suspended";
  const now = Date.now();
  const diff = due - now;
  if (diff <= 0) return "Due";
  const days = Math.round(diff / DAY);
  if (diff < DAY) return "Today";
  if (days === 1) return "Tomorrow";
  if (days < 30) return `${days}d`;
  if (days < 365) return `${(days / 30).toFixed(0)}mo`;
  return `${(days / 365).toFixed(1)}y`;
}

export function intervalLabel(days: number): string {
  if (days <= 0) return "—";
  if (days < 30) return `${days}d`;
  if (days < 365) return `${(days / 30).toFixed(1)}mo`;
  return `${(days / 365).toFixed(1)}y`;
}

export const STATE_LABEL: Record<string, string> = {
  new: "New",
  learning: "Learning",
  relearning: "Relearning",
  review: "Review",
  suspended: "Suspended",
};

export function pluralize(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? "" : "s"}`;
}

<script lang="ts">
  let {
    name,
    size = 20,
    stroke = 1.6,
  }: { name: string; size?: number; stroke?: number } = $props();

  const paths: Record<string, string> = {
    study: `<rect x="3" y="4" width="14" height="16" rx="2.5"/><path d="M7 9h6M7 13h4"/><path d="M17 7l3 1.2a2 2 0 0 1 1.2 2.5l-2.8 8"/>`,
    decks: `<path d="M12 3 3 7.5 12 12l9-4.5L12 3Z"/><path d="M3 12.5 12 17l9-4.5"/><path d="M3 16.5 12 21l9-4.5" opacity=".5"/>`,
    browse: `<rect x="3" y="4" width="18" height="16" rx="2.5"/><path d="M3 9h18M9 9v11"/>`,
    stats: `<path d="M5 20V11M12 20V4M19 20v-6"/>`,
    settings: `<path d="M5 7h14M5 12h14M5 17h14"/><circle cx="9" cy="7" r="2" fill="var(--bg)"/><circle cx="15" cy="12" r="2" fill="var(--bg)"/><circle cx="8" cy="17" r="2" fill="var(--bg)"/>`,
    plus: `<path d="M12 5v14M5 12h14"/>`,
    search: `<circle cx="11" cy="11" r="7"/><path d="m20 20-3.2-3.2"/>`,
    sync: `<path d="M20 11a8 8 0 0 0-14-4.5L4 8"/><path d="M4 4v4h4"/><path d="M4 13a8 8 0 0 0 14 4.5L20 16"/><path d="M20 20v-4h-4"/>`,
    check: `<path d="M5 12.5 10 17 19 7"/>`,
    x: `<path d="M6 6l12 12M18 6 6 18"/>`,
    trash: `<path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13"/>`,
    edit: `<path d="M4 20h4L19 9l-4-4L4 16v4Z"/><path d="M14 6l4 4"/>`,
    chevron: `<path d="m9 6 6 6-6 6"/>`,
    flame: `<path d="M12 3c1 3-1 4-1 6a3 3 0 0 0 6 0c0-1-.5-2-1-3 2 1.5 3 4 3 6a6 6 0 0 1-12 0c0-3 3-4 5-9Z"/>`,
    clock: `<circle cx="12" cy="12" r="8"/><path d="M12 8v4l3 2"/>`,
    moon: `<path d="M20 14A8 8 0 0 1 10 4a8 8 0 1 0 10 10Z"/>`,
    folder: `<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"/>`,
    sparkle: `<path d="M12 4l1.6 4.4L18 10l-4.4 1.6L12 16l-1.6-4.4L6 10l4.4-1.6L12 4Z"/>`,
    arrowLeft: `<path d="M19 12H5M11 6l-6 6 6 6"/>`,
    eye: `<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/>`,
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {@html paths[name] ?? ""}
</svg>

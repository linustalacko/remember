# Security Policy

## Supported versions

Remember is pre-1.0. Security fixes land on the latest release and `main`.

## Reporting a vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately via GitHub's
[**Report a vulnerability**](https://github.com/linustalacko/remember/security/advisories/new)
form (the **Security** tab → "Report a vulnerability"). I'll acknowledge your
report, work with you on a fix, and credit you if you'd like.

## Threat model

Remember is a local-first desktop app. The most important boundaries:

- **Untrusted card content.** Flashcard notes — especially decks imported from a
  third party's `.anki2` file — can contain arbitrary HTML. All note HTML is
  sanitized with DOMPurify before it is rendered, and the webview runs under a
  strict Content-Security-Policy (see `src-tauri/tauri.conf.json`). Imported
  files are validated as SQLite before being opened.
- **Sync credentials.** The optional Turso auth token is stored in a
  device-local file (owner-only on macOS/Linux) and is never included in synced
  data or returned to the frontend.
- **Out of scope.** An attacker who already has local code-execution or your file
  permissions on the machine (they can read any of your files regardless), and
  the security of the third-party services you choose to connect (Turso).

If you find a way for imported or synced content to execute code, escape the
webview's CSP, or exfiltrate the sync token, that's exactly the kind of report
I want to hear about.

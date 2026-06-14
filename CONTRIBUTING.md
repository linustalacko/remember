# Contributing to Remember

Thanks for your interest in improving Remember! This guide covers how to get set
up and what to expect.

## Development setup

**Prerequisites**

- [Node.js](https://nodejs.org) 18+ and [pnpm](https://pnpm.io)
- The [Rust toolchain](https://rustup.rs)
- Tauri's platform prerequisites — see the
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/)

```bash
pnpm install
pnpm tauri dev          # run the app with hot reload
```

## Checks before you open a PR

```bash
pnpm check                       # svelte-check (types + Svelte)
cargo check  --manifest-path src-tauri/Cargo.toml
cargo test   --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
```

The same checks run in CI on every pull request.

## Code style

- **Rust** — please run `cargo fmt` before committing.
- **Frontend** — match the surrounding TypeScript/Svelte style. Indentation and
  newline rules are captured in [`.editorconfig`](.editorconfig).
- Keep changes focused; one logical change per PR makes review easier.

## Working with the scheduler and importer

The scheduling logic lives in `src-tauri/src/srs.rs` and the Anki importer in
`src-tauri/src/importer.rs`. If you change scheduling behavior, please describe
the intended interval math in your PR — it's the part most likely to surprise
users. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for an overview.

## Reporting bugs and requesting features

Use the [issue templates](https://github.com/linustalacko/remember/issues/new/choose).
For anything security-sensitive, follow [SECURITY.md](SECURITY.md) instead of
opening a public issue.

By contributing, you agree that your contributions are licensed under the
project's [MIT License](LICENSE).

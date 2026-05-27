# Synergism Forkd

[![rust-ci](https://github.com/MaddisonM79/synergism_forkd/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/MaddisonM79/synergism_forkd/actions/workflows/rust-ci.yml)
[![License](https://img.shields.io/github/license/MaddisonM79/synergism_forkd)](LICENSE)
[![Last commit](https://img.shields.io/github/last-commit/MaddisonM79/synergism_forkd/main)](https://github.com/MaddisonM79/synergism_forkd/commits/main)
[![break-eternity-rs](https://img.shields.io/crates/v/break-eternity-rs?label=break-eternity-rs)](https://crates.io/crates/break-eternity-rs)

A Rust rewrite of the TypeScript idle game **Synergism**. Forked, renamed, and ported one mechanic at a time.

## Layout

```
crates/
  synergismforkd_bignum/      # break-eternity-rs wrapper (Decimal)
  synergismforkd_common/      # shared types
  synergismforkd_logic/       # headless game logic
  synergismforkd_save/        # save format + migrations
  synergismforkd_ui/          # Dioxus components (platform-agnostic)
  synergismforkd_ui_web/      # WASM browser entry point
  synergismforkd_ui_desktop/  # Tauri shell (Win/Mac/Linux)
  synergismforkd_testkit/     # fixtures + sim runner + synergismforkd-sim CLI
assets/                       # translations, pictures, sounds
legacy/
  original/                   # frozen pre-split TS, reference only
  core_split/                 # current packages/ snapshot (TS), reference only
```

The legacy TS folders are **not maintained** — they live in the repo so the Rust port has the original source to reference while porting mechanics one by one.

## Quickstart

Requires Rust stable (the toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml); rustup will auto-install).

```sh
cargo build --workspace
cargo test --workspace
cargo run -p synergismforkd_testkit --bin synergismforkd-sim
```

WASM browser build:

```sh
cargo build -p synergismforkd_ui_web --target wasm32-unknown-unknown
```

## Status

Bare-bones scaffold. Most crates ship a single placeholder function; the real work is porting mechanics from `legacy/core_split/packages/logic/` and standing up the Dioxus UI tree. See [CLAUDE.md](CLAUDE.md) for crate boundary rules and porting guidance.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md).

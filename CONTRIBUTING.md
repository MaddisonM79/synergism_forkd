# Contributing

Thanks for the interest. This is the Rust port of the TypeScript idle game Synergism. The legacy TS sources live in `legacy/` for reference; active development happens in the Cargo workspace at the repo root.

## Prerequisites

- **Rust** stable — the channel, components (`clippy`, `rustfmt`), and the `wasm32-unknown-unknown` target are pinned in [rust-toolchain.toml](rust-toolchain.toml). `rustup` will auto-install on first `cargo` invocation, so you typically don't need to install anything by hand.
- **git** — https://git-scm.com/downloads

## Recommended editor

- [Zed](https://zed.dev) — first-class `rust-analyzer` integration out of the box, fast on large workspaces. Open the repo root and Zed indexes the workspace automatically.
- VSCode fallback — install the official `rust-analyzer` extension.

## Workflow

1. Fork this repository.
2. Clone your fork:
   ```sh
   git clone https://github.com/<USERNAME>/synergism_forkd
   cd synergism_forkd
   ```
3. Create a branch:
   ```sh
   git checkout -b my-branch-name
   ```
4. Build and test:
   ```sh
   cargo build --workspace
   cargo test --workspace
   ```
5. Format and lint before committing — CI rejects warnings:
   ```sh
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   ```
6. Commit and push, then open a pull request against `main`.

### Headless sim

The sim runner exercises the tick loop end-to-end without a UI:

```sh
cargo run -p synergismforkd_testkit --bin synergismforkd-sim
```

### WASM browser build

```sh
cargo build -p synergismforkd_ui_web --target wasm32-unknown-unknown
```

## Pull request titles

PR titles follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format and are checked by the `pr-title-lint` workflow:

```
type(optional-scope): short lowercase subject
```

Allowed types: `feat`, `fix`, `chore`, `ci`, `docs`, `perf`, `refactor`, `revert`, `security`, `style`, `test`, `ts`, `ux`.

Examples:

- `fix: prevent rune 7 race during singularity`
- `ux: shorten notification slide-in to 200ms`
- `feat(ants): add Reborn-ELO tranche 11`

Since merges are squashed, the PR title becomes the commit message on `main`. Individual commits on your feature branch can use any message you like.

## Save format changes

Changes that add or modify fields on the game state affect every player's savefile size and migration path. **Before adding fields to `crates/synergismforkd_logic/src/state/` slices, please open an issue first** to discuss the schema impact, then mirror the change in `crates/synergismforkd_save/` with a migration if the schema version bumps. See the "Save system" section of [CLAUDE.md](CLAUDE.md) for the full rule.

## Releases

Desktop builds are produced by [.github/workflows/desktop-release.yml](.github/workflows/desktop-release.yml) on tagged pushes matching `v*.*.*`. Pushing to `main` does not release — version bumps are explicit.

## Further reading

- [CLAUDE.md](CLAUDE.md) — crate boundary rules, save-system invariants, porting guidance.
- [docs/audits/STATE_AUDIT.md](docs/audits/STATE_AUDIT.md) — current state-slice audit and porting priorities.

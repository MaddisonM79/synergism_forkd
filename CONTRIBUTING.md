# Contributing

## Prerequisites

Before running any of the commands below, make sure you have installed:

- NodeJS >= 22.21.0 — https://nodejs.org/en/
- git — https://git-scm.com/downloads

## Recommended

- VSCode — https://code.visualstudio.com/Download

## Workflow

1. Fork this repository.
2. Clone your fork:
   ```sh
   git clone https://github.com/<USERNAME>/unknown_game
   cd unknown_game
   ```
3. Install dependencies:
   ```sh
   npm install
   ```
4. Switch to a new branch:
   ```sh
   git checkout -b my-branch-name
   ```
5. Start the dev server:
   ```sh
   node --run dev
   ```
6. Make your changes and verify them in the browser.
7. Type-check:
   ```sh
   node --run check:tsc
   ```
8. Lint:
   ```sh
   node --run lint
   node --run csslint
   ```
9. Stage and commit:
   ```sh
   git add <files>
   git commit -m "short description"
   ```
10. Push and open a pull request against this repository.

## Pull request titles

PR titles follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format and are checked by the `pr-title-lint` workflow:

```
type(optional-scope): short lowercase subject
```

Allowed types: `feat`, `fix`, `chore`, `ci`, `docs`, `perf`, `refactor`, `revert`, `security`, `style`, `test`, `ts`, `ux`.

Examples:

- `fix: prevent rune 7 race during singularity`
- `ux: shorten notification slide-in to 200ms`
- `ci: install deep-object-diff for translation check`
- `feat(ants): add Reborn-ELO tranche 11`

Since merges are squashed, the PR title becomes the commit message on `main`. Individual commits on your feature branch can use any message you like.

## Save format changes

Changes that add or modify fields on the `player` object affect every player's savefile size and migration path. **Before making such a change, please open an issue first** to discuss the schema impact.

See also [CLAUDE.md](CLAUDE.md) for the agent-facing contribution rules (i18n, DOM caching, Steam/Electron gating, etc.).

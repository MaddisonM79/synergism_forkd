# Contributing

## Prerequisites

Before running any of the commands below, make sure you have installed:

- NodeJS >= 24.0.0 — https://nodejs.org/en/
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

## Save format changes

Changes that add or modify fields on the `player` object affect every player's savefile size and migration path. **Before making such a change, please open an issue first** to discuss the schema impact.

See also [CLAUDE.md](CLAUDE.md) for the agent-facing contribution rules (i18n, DOM caching, Steam/Electron gating, etc.).

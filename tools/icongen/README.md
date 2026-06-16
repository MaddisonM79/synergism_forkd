# icongen — painted icon generation

Generates the game's **painted (raster) icons** with OpenAI `gpt-image-1` and
writes transparent-background PNGs into `assets/pictures/<category>/<id>.png`.

This is an **offline asset tool**, not part of the Rust build. It lives at the
repo root (no crate-boundary concerns) and is run by hand when icons change.

## Setup

```sh
pip install -r tools/icongen/requirements.txt
export OPENAI_API_KEY=sk-...        # you set this; the script only reads it
```

The key is read from the environment and never stored by the script.

## Use

```sh
# Pilot: lock the style on a few icons before spending on the whole set.
python tools/icongen/generate.py --only coins,quarks,goldenquarks,ambrosia,ach1,ach2

# Full manifest.
python tools/icongen/generate.py

# Regenerate even where outputs already exist.
python tools/icongen/generate.py --force

# Extra options
python tools/icongen/generate.py --sizes 256,128,64   # output px (first is canonical)
```

Outputs: `crates/synergismforkd_ui/assets/pictures/<category>/<id>.png`
(256px master only — the UI downscales it crisply at render time). They live
inside the UI crate so Dioxus' `asset!()` macro bundles them. Existing files
are skipped unless `--force`. Override the destination with `--out`, or add
extra downscaled sizes with `--sizes 256,64`.

## Files

- `manifest.json` — every icon's `id`, `category`, and `prompt` (the subject).
  Add entries here to grow the set; colors are quoted from the dark-theme
  `--res-*` tokens in `crates/synergismforkd_ui/assets/styles/themes.css`.
- `style.txt` — the shared house-style preamble prepended to every prompt.
  **Tune this first** during the pilot — it controls the whole set's look.
- `generate.py` — the generator + Pillow post-process (alpha-trim, center,
  downscale).

## Workflow

1. Run the pilot subset, inspect the PNGs at 64px.
2. Edit `style.txt` until the look is right; re-run with `--force`.
3. Fill out `manifest.json` and run the full batch.
4. Wire icons into the UI via the raster path in
   `crates/synergismforkd_ui/src/components/icon.rs`.

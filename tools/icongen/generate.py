#!/usr/bin/env python3
"""Generate painted game icons via OpenAI gpt-image-1.

Reads a manifest of icon specs, prepends the shared house-style preamble
(style.txt), generates one transparent-background image per icon, then
post-processes (alpha-trim → center → downscale) into assets/pictures/.

The OpenAI key is read from the environment — this script never stores it:

    export OPENAI_API_KEY=sk-...
    python tools/icongen/generate.py --only coins,quarks,goldenquarks,ambrosia,ach1,ach2
    python tools/icongen/generate.py            # whole manifest
    python tools/icongen/generate.py --force    # regenerate even if files exist

Requires: openai>=1.0, Pillow  (pip install -r tools/icongen/requirements.txt)
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent
DEFAULT_MANIFEST = HERE / "manifest.json"
DEFAULT_STYLE = HERE / "style.txt"
# Output lives inside the UI crate so Dioxus' `asset!()` macro bundles it
# (paths resolve relative to crates/synergismforkd_ui/, and `dx serve` watches
# that assets dir). Override with --out.
DEFAULT_OUT = REPO_ROOT / "crates" / "synergismforkd_ui" / "assets" / "pictures"

# Output sizes (px). The first is canonical (<id>.png); any extra sizes are
# written as <id>.<size>.png. Generation is always at 1024 then downscaled.
# We ship only the 256px master — the UI renders icons at ~22-54px and the
# browser downscales the 256px crisply (sharper than a baked-in small variant).
DEFAULT_SIZES = [256]
GEN_SIZE = "1024x1024"
MODEL = "gpt-image-1"


def eprint(*a: object) -> None:
    print(*a, file=sys.stderr)


def load_manifest(path: Path) -> list[dict]:
    data = json.loads(path.read_text())
    icons = data.get("icons", [])
    if not icons:
        eprint(f"manifest {path} has no 'icons'")
        sys.exit(1)
    return icons


def primary_path(out_root: Path, spec: dict, size: int) -> Path:
    return out_root / spec["category"] / f"{spec['id']}.png"


def variant_path(out_root: Path, spec: dict, size: int) -> Path:
    return out_root / spec["category"] / f"{spec['id']}.{size}.png"


def postprocess_and_save(raw_png: bytes, spec: dict, sizes: list[int], out_root: Path) -> None:
    from io import BytesIO

    from PIL import Image

    img = Image.open(BytesIO(raw_png)).convert("RGBA")

    # Alpha-trim to the painted subject, then re-center on a square canvas with
    # a small uniform margin so every icon is consistently framed.
    bbox = img.getbbox()
    if bbox:
        img = img.crop(bbox)
    side = max(img.size)
    margin = round(side * 0.06)
    canvas = Image.new("RGBA", (side + 2 * margin, side + 2 * margin), (0, 0, 0, 0))
    canvas.paste(img, (margin + (side - img.width) // 2, margin + (side - img.height) // 2))

    for i, size in enumerate(sizes):
        dst = primary_path(out_root, spec, size) if i == 0 else variant_path(out_root, spec, size)
        dst.parent.mkdir(parents=True, exist_ok=True)
        canvas.resize((size, size), Image.LANCZOS).save(dst)
        print(f"    wrote {dst.relative_to(REPO_ROOT)}")


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate painted game icons (gpt-image-1).")
    ap.add_argument("--only", help="comma-separated icon ids to generate (default: all)")
    ap.add_argument("--force", action="store_true", help="regenerate even if the output exists")
    ap.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    ap.add_argument("--style", type=Path, default=DEFAULT_STYLE)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--sizes", help="comma-separated output px (default: 256,64)")
    args = ap.parse_args()

    if not os.environ.get("OPENAI_API_KEY"):
        eprint("OPENAI_API_KEY is not set. Run:  export OPENAI_API_KEY=sk-...")
        return 2

    try:
        from openai import OpenAI
    except ImportError:
        eprint("Missing deps. Run:  pip install -r tools/icongen/requirements.txt")
        return 2

    sizes = [int(s) for s in args.sizes.split(",")] if args.sizes else DEFAULT_SIZES
    style = args.style.read_text().rstrip() + " "
    icons = load_manifest(args.manifest)
    wanted = set(args.only.split(",")) if args.only else None
    if wanted:
        icons = [s for s in icons if s["id"] in wanted]
        missing = wanted - {s["id"] for s in icons}
        if missing:
            eprint(f"warning: ids not in manifest, skipped: {sorted(missing)}")

    client = OpenAI()
    generated = skipped = failed = 0

    for spec in icons:
        dst = primary_path(args.out, spec, sizes[0])
        if dst.exists() and not args.force:
            print(f"[skip] {spec['id']} (exists; --force to overwrite)")
            skipped += 1
            continue

        prompt = style + spec["prompt"]
        print(f"[gen ] {spec['id']} ({spec['category']})")
        try:
            resp = client.images.generate(
                model=MODEL,
                prompt=prompt,
                size=GEN_SIZE,
                background="transparent",
                n=1,
            )
            raw = base64.b64decode(resp.data[0].b64_json)
            postprocess_and_save(raw, spec, sizes, args.out)
            generated += 1
        except Exception as exc:  # noqa: BLE001 — report and continue the batch
            eprint(f"    FAILED {spec['id']}: {exc}")
            failed += 1

    print(f"\nDone. generated={generated} skipped={skipped} failed={failed} "
          f"(of {len(icons)} requested)")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())

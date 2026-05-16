#!/usr/bin/env python3
"""Download ambient sound clips for the dashboard's Ambience widget.

URLs are read from `.env` (AMBIENT_SOUND_1..4); each slot maps to a fixed
filename in `static/sounds/`. Re-running is safe -files are skipped unless
the URL has changed since the last download (tracked in `.manifest.json`).

Requires `yt-dlp` (in `requirements.txt`) and `ffmpeg` on PATH.

Usage:
  python scripts/download_sounds.py            # download missing / changed
  python scripts/download_sounds.py --force    # delete and re-download all
"""
import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
ENV_FILE = PROJECT_ROOT / ".env"
SOUNDS_DIR = PROJECT_ROOT / "static" / "sounds"
MANIFEST = SOUNDS_DIR / ".manifest.json"

# Fallback URLs -used when the corresponding env var is missing.
DEFAULTS = {
    "AMBIENT_SOUND_1": "https://www.youtube.com/watch?v=uiMXGIG_DQo",
    "AMBIENT_SOUND_2": "https://www.youtube.com/watch?v=mPZkdNFkNps",
    "AMBIENT_SOUND_3": "https://www.youtube.com/watch?v=p8PaZdSIIuk",
    "AMBIENT_SOUND_4": "https://www.youtube.com/watch?v=JkynE2qDPGw",
}

# Slot → filename slug (matches what Ambience.svelte expects).
SLOTS = [
    ("AMBIENT_SOUND_1", "cafe"),
    ("AMBIENT_SOUND_2", "rain"),
    ("AMBIENT_SOUND_3", "medieval"),
    ("AMBIENT_SOUND_4", "cyberpunk"),
]


def env_var(name: str) -> str:
    val = os.environ.get(name, "")
    if val:
        return val
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith(f"{name}="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    return ""


def load_manifest() -> dict:
    if MANIFEST.exists():
        try:
            return json.loads(MANIFEST.read_text(encoding="utf-8"))
        except Exception:
            return {}
    return {}


def save_manifest(m: dict) -> None:
    MANIFEST.write_text(json.dumps(m, indent=2), encoding="utf-8")


def download_one(url: str, slug: str) -> bool:
    out = SOUNDS_DIR / f"{slug}.mp3"
    print(f"  Downloading {slug}…")
    result = subprocess.run(
        [
            "yt-dlp",
            "-x",
            "--audio-format", "mp3",
            "--audio-quality", "64K",
            "--download-sections", "*0:00-10:00",
            "-o", str(out),
            url,
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        size = out.stat().st_size / 1024 / 1024
        print(f"  + {slug}.mp3 ({size:.1f} MB)")
        return True
    print(f"  x {slug} failed:\n{result.stderr[-500:]}", file=sys.stderr)
    return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--force", action="store_true", help="Delete and re-download all sounds")
    args = parser.parse_args()

    SOUNDS_DIR.mkdir(parents=True, exist_ok=True)
    manifest = load_manifest()

    for env_name, slug in SLOTS:
        url = env_var(env_name) or DEFAULTS[env_name]
        out = SOUNDS_DIR / f"{slug}.mp3"

        if args.force and out.exists():
            out.unlink()

        if out.exists():
            if manifest.get(slug) == url:
                print(f"  {slug}.mp3 up-to-date -skipping")
                continue
            if slug in manifest:
                print(f"  {slug}.mp3 URL changed -re-downloading")
                out.unlink()
            else:
                # Existing file with no manifest entry -assume it's fine, just record it.
                manifest[slug] = url
                continue

        if download_one(url, slug):
            manifest[slug] = url

    save_manifest(manifest)
    print("\nDone. Files in static/sounds/")
    return 0


if __name__ == "__main__":
    sys.exit(main())

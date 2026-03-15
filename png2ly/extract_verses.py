#!/usr/bin/env python3
"""Extract additional verse lyrics from psalm PNGs and create lyrics_N.ly files.

For each psalm, extracts lyrics from photos/psalm*/2.png, 3.png, etc.
(concatenating a/b parts if needed). Creates lilypond/psalm*/lyrics_N.ly
then renders SVG using notes.ly + lyrics_N.ly via render_ly.
"""

import argparse
import glob
import os
import re
import subprocess
import sys
import time
from multiprocessing.pool import ThreadPool

import cv2
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)

sys.path.insert(0, SCRIPT_DIR)
from png2ly import run_claude, build_lyrics_ly
from render_ly import render_svg


def concatenate_images_vertically(paths, output_path):
    """Concatenate multiple PNG images vertically."""
    images = [cv2.imread(p) for p in paths]
    if any(img is None for img in images):
        raise ValueError(f"Could not read one of: {paths}")
    max_w = max(img.shape[1] for img in images)
    padded = []
    for img in images:
        h, w = img.shape[:2]
        if w < max_w:
            pad = np.ones((h, max_w - w, 3), dtype=np.uint8) * 255
            img = np.hstack([img, pad])
        padded.append(img)
    combined = np.vstack(padded)
    cv2.imwrite(output_path, combined)
    return output_path


def ocr_image(img_path):
    """Run Tesseract OCR on an image and return the text."""
    result = subprocess.run(
        ["tesseract", img_path, "-"],
        capture_output=True,
        text=True,
        timeout=30,
    )
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def text_to_lyricmode(text, num_notes_per_line):
    """Convert plain text lyrics to LilyPond lyricmode format using Claude."""
    notes_info = ", ".join(str(n) for n in num_notes_per_line)
    prompt = (
        "Convert these hymn lyrics to LilyPond \\lyricmode format.\n\n"
        f"Lyrics:\n{text}\n\n"
        "Rules:\n"
        "- Use ' -- ' (space-dash-dash-space) between syllables of the same word\n"
        "- Use spaces between separate words\n"
        "- Do not include verse numbers\n"
        "- Include all punctuation as it appears\n"
        f"- The melody has {len(num_notes_per_line)} lines with this many notes per line: {notes_info}\n"
        "- Each syllable should align with one note. If a syllable spans multiple notes "
        "(melisma), use '_' for the extra notes after the syllable.\n"
        "- Output nothing except the lyricmode content (no \\lyricmode wrapper, no explanation)"
    )
    text = run_claude(prompt)
    if not text:
        return None
    # Strip markdown code fences
    if "```" in text:
        blocks = text.split("```")
        for i in range(len(blocks) - 1, 0, -1):
            candidate = blocks[i].strip()
            for prefix in ["lilypond", "ly", "text"]:
                if candidate.startswith(prefix):
                    candidate = candidate[len(prefix):].strip()
            if candidate and "--" in candidate:
                return candidate
    # Filter out explanation lines
    lines = []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("-") or line.startswith("*") or ":" in line[:20]:
            continue
        lines.append(line)
    return "\n".join(lines) if lines else text


def count_notes_per_line(notes_path):
    """Count pitched notes (not rests) per line from a notes.ly file."""
    with open(notes_path) as f:
        content = f.read()

    counts = []
    for segment in content.split("\\break"):
        notes = re.findall(r'(?<![a-z])[a-g](?:is|es)?[0-9\',]*', segment)
        if notes:
            counts.append(len(notes))
    return counts


def find_verse_pngs(psalm_photo_dir):
    """Find all verse PNGs (2.png, 3.png, 2a.png+2b.png, etc.) in a psalm dir.

    Returns list of (verse_num, [png_paths]).
    """
    verses = {}

    for f in os.listdir(psalm_photo_dir):
        # Single files: 2.png, 3.png
        m = re.match(r'^(\d+)\.png$', f)
        if m:
            num = int(m.group(1))
            if num >= 2:
                verses[num] = [os.path.join(psalm_photo_dir, f)]

    for f in sorted(os.listdir(psalm_photo_dir)):
        # Multi-part: 2a.png, 2b.png
        m = re.match(r'^(\d+)([a-z])\.png$', f)
        if m:
            num = int(m.group(1))
            if num >= 2:
                if num not in verses:
                    verses[num] = []
                verses[num].append(os.path.join(psalm_photo_dir, f))

    return sorted(verses.items())


def process_verse(args):
    """Process a single verse. Returns (psalm_name, verse_num, status, elapsed)."""
    psalm_name, verse_num, png_paths, out_dir = args
    t0 = time.time()

    try:
        notes_path = os.path.join(out_dir, "notes.ly")
        notes_per_line = count_notes_per_line(notes_path)
        if not notes_per_line:
            return (psalm_name, verse_num, "FAILED: no notes found", time.time() - t0)

        # Handle a/b concatenation
        if len(png_paths) > 1:
            concat_path = os.path.join(out_dir, f"{verse_num}.png")
            concatenate_images_vertically(png_paths, concat_path)
            img_path = concat_path
        else:
            img_path = png_paths[0]

        # OCR then format with Claude
        raw_text = ocr_image(img_path)
        if not raw_text:
            return (psalm_name, verse_num, "FAILED: OCR failed", time.time() - t0)

        lyrics = text_to_lyricmode(raw_text, notes_per_line)
        if not lyrics:
            return (psalm_name, verse_num, "FAILED: no lyrics", time.time() - t0)

        # Write lyrics file
        lyrics_path = os.path.join(out_dir, f"lyrics_{verse_num}.ly")
        with open(lyrics_path, "w") as f:
            f.write(build_lyrics_ly(lyrics))

        # Read composer
        composer = None
        composer_path = os.path.join(out_dir, "composer.txt")
        if os.path.exists(composer_path):
            with open(composer_path) as f:
                composer = f.read().strip()

        # Render SVG
        svg_path = os.path.join(out_dir, f"{verse_num}.svg")
        render_svg(notes_path, lyrics_path, svg_path, composer=composer)

        return (psalm_name, verse_num, "OK", time.time() - t0)
    except Exception as e:
        return (psalm_name, verse_num, f"FAILED: {e}", time.time() - t0)


def main():
    parser = argparse.ArgumentParser(description="Extract verse lyrics and create .ly files")
    parser.add_argument("-n", "--limit", type=int, help="Max number of verses to process")
    parser.add_argument("-j", "--jobs", type=int, default=4, help="Parallel workers (default: 4)")
    parser.add_argument("--psalm", help="Process only this psalm (e.g. psalm6)")
    args = parser.parse_args()

    photos_dir = os.path.join(REPO_DIR, "photos")
    lilypond_dir = os.path.join(REPO_DIR, "lilypond")

    work = []
    if args.psalm:
        psalm_dirs = [os.path.join(photos_dir, args.psalm)]
    else:
        psalm_dirs = sorted(glob.glob(os.path.join(photos_dir, "psalm*")))

    for psalm_photo_dir in psalm_dirs:
        psalm_name = os.path.basename(psalm_photo_dir)
        out_dir = os.path.join(lilypond_dir, psalm_name)
        notes_path = os.path.join(out_dir, "notes.ly")

        if not os.path.exists(notes_path):
            continue

        verses = find_verse_pngs(psalm_photo_dir)
        for verse_num, png_paths in verses:
            lyrics_path = os.path.join(out_dir, f"lyrics_{verse_num}.ly")
            if os.path.exists(lyrics_path):
                continue
            work.append((psalm_name, verse_num, png_paths, out_dir))

    if args.limit:
        work = work[:args.limit]

    print(f"Found {len(work)} verses to process")
    t_total = time.time()

    with ThreadPool(args.jobs) as pool:
        for psalm_name, verse_num, status, elapsed in pool.imap_unordered(process_verse, work):
            print(f"  {psalm_name} v{verse_num}: {status} [{elapsed:.0f}s]")

    print(f"Done. Total: {time.time() - t_total:.0f}s")


if __name__ == "__main__":
    main()

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
from png2ly import run_claude, build_lyrics_ly, detect_staff_systems, extract_lyrics_with_claude
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


def get_note_lines(notes_path):
    """Get the pitched note content (excluding rests) for each line from a notes.ly file."""
    with open(notes_path) as f:
        all_lines = f.readlines()

    result = []
    for i, line in enumerate(all_lines):
        if re.match(r'\s*%\s*Line\s+\d+', line):
            # Next line has the notes
            if i + 1 < len(all_lines):
                note_line = all_lines[i + 1]
                notes = re.findall(r'(?<![a-z])[a-g](?:is|es)?[0-9\',]+', note_line)
                result.append(" ".join(notes))
    return result


def split_line_syllables(text_line, note_line, note_count):
    """Ask sonnet to split one line of text into syllables matching note count."""
    prompt = (
        f"Split this hymn lyric line into exactly {note_count} syllables for LilyPond.\n\n"
        f"Text: {text_line}\n"
        f"Notes: {note_line}\n"
        f"Required syllable count: {note_count}\n\n"
        "Rules:\n"
        "- Use ' -- ' (space-dash-dash-space) between syllables of the SAME word\n"
        "- Use spaces between DIFFERENT words\n"
        "- Each syllable gets one note\n"
        f"- Count must be exactly {note_count}\n"
        "- Standalone punctuation like dashes (– —) should be attached to the preceding word, not counted as a separate syllable\n"
        "- Include punctuation (commas, periods, semicolons, etc.) attached to the syllable, not as separate tokens\n"
        "- Output ONLY the syllable-split text, nothing else"
    )
    result = run_claude(prompt, model="sonnet", timeout=30)
    if not result:
        return text_line
    # Clean up — take first non-empty line that isn't explanation
    for line in result.splitlines():
        line = line.strip()
        if line and not line.startswith("-") and ":" not in line[:15]:
            # Remove standalone dashes (not syllable separators)
            line = re.sub(r'\s+[–—]\s*$', '', line)
            line = re.sub(r'\s+[–—]\s+', ' ', line)
            return line
    return result.strip()


def extract_lyrics_from_image(img_path, num_notes_per_line, note_lines, raw_text_path=None):
    """Extract lyrics from an image.

    If the image has staff lines (music notation), extract lyrics directly
    using Claude (same approach as png2ly for verse 1).
    Otherwise, use two stages: haiku OCR + sonnet syllable splitting.
    """
    # Check if image has music notation
    try:
        systems, _ = detect_staff_systems(img_path)
        has_staves = len(systems) >= 3
    except (ValueError, Exception):
        has_staves = False

    if has_staves:
        # Extract lyrics directly from music image (like png2ly)
        return extract_lyrics_with_claude(img_path, num_notes_per_line)

    # Text-only image: two-stage approach
    # Stage 1: Extract raw text with haiku
    ocr_prompt = (
        f"Read the image at {img_path} and then: "
        "Extract ALL the text from this image. "
        "Keep each line of text on a separate line. "
        "Output only the text, nothing else. "
        "Do not include verse numbers."
    )
    raw_text = run_claude(ocr_prompt, model="haiku")
    if raw_text and raw_text_path:
        with open(raw_text_path, "w") as f:
            f.write(raw_text)
    if not raw_text:
        return None

    # Stage 2: Split each line into syllables
    text_lines = [l.strip() for l in raw_text.splitlines() if l.strip()]

    # Match text lines to note lines
    result_lines = []
    for i, note_count in enumerate(num_notes_per_line):
        if i < len(text_lines):
            text_line = text_lines[i]
        else:
            text_line = ""
        note_line = note_lines[i] if i < len(note_lines) else ""
        split = split_line_syllables(text_line, note_line, note_count)
        result_lines.append(split)

    return "\n".join(result_lines)


def count_notes_per_line(notes_path):
    """Count pitched notes (not rests) per line from a notes.ly file."""
    return [len(nl.split()) for nl in get_note_lines(notes_path)]


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
            if num >= 1:
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
        note_lines = get_note_lines(notes_path)
        if not notes_per_line:
            return (psalm_name, verse_num, "FAILED: no notes found", time.time() - t0)

        # Handle a/b concatenation
        if len(png_paths) > 1:
            concat_path = os.path.join(out_dir, f"{verse_num}.png")
            concatenate_images_vertically(png_paths, concat_path)
            img_path = concat_path
        else:
            img_path = png_paths[0]

        # Extract lyrics: haiku OCR then sonnet syllable splitting per line
        raw_text_path = os.path.join(out_dir, f"raw_text_{verse_num}.txt")
        lyrics = extract_lyrics_from_image(img_path, notes_per_line, note_lines, raw_text_path)
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

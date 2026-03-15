#!/usr/bin/env python3
"""Run png2ly on all psalm images matching photos/psalm*/1.png or 1a.png+1b.png."""

import argparse
import glob
import os
import shutil
import subprocess
import sys
import tempfile
import time
from multiprocessing.pool import ThreadPool

import cv2
import numpy as np

# Add parent dir so we can find audiveris
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)

sys.path.insert(0, SCRIPT_DIR)
from png2ly import (
    detect_staff_systems,
    crop_lines,
    run_audiveris,
    extract_mxl,
    parse_musicxml,
    detect_key_fifths,
    apply_key_signature,
    extract_composer_with_claude,
    extract_lyrics_with_claude,
    build_notes_ly,
    build_lyrics_ly,
)
from render_ly import render_svg


def concatenate_images_vertically(paths, output_path):
    """Concatenate multiple PNG images vertically."""
    images = [cv2.imread(p) for p in paths]
    if any(img is None for img in images):
        raise ValueError(f"Could not read one of: {paths}")
    # Match widths by padding narrower images
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


def find_psalm_input(psalm_dir):
    """Determine the input PNG for a psalm directory.

    Returns (png_path, needs_concat, parts) where:
    - If 1.png exists: (path_to_1.png, False, [])
    - If 1a.png+1b.png exist: (None, True, [1a.png, 1b.png, ...])
    """
    single = os.path.join(psalm_dir, "1.png")
    if os.path.exists(single):
        return single, False, []

    # Look for 1a.png, 1b.png, etc.
    parts = sorted(glob.glob(os.path.join(psalm_dir, "1[a-z].png")))
    if parts:
        return None, True, parts

    return None, False, []


def process_psalm(args):
    """Run the full png2ly pipeline on a single image. Returns (psalm_name, status, elapsed)."""
    png_path, out_ly, concat_save_path = args
    psalm_name = os.path.basename(os.path.dirname(
        out_ly if not concat_save_path else out_ly
    ))
    t0 = time.time()

    tmpdir = tempfile.mkdtemp(prefix="png2ly_")
    try:
        systems, img_height = detect_staff_systems(png_path)
        line_paths = crop_lines(png_path, systems, img_height, tmpdir)

        all_line_data = []
        for line_path in line_paths:
            try:
                mxl_path = run_audiveris(line_path, tmpdir)
                xml_path = extract_mxl(mxl_path, tmpdir)
                notes, key_fifths = parse_musicxml(xml_path)
                all_line_data.append((notes, key_fifths))
            except Exception as e:
                all_line_data.append(([], None))

        global_key = detect_key_fifths(all_line_data)
        all_lines = []
        for notes, _ in all_line_data:
            all_lines.append(apply_key_signature(notes, global_key))

        composer = extract_composer_with_claude(png_path)
        num_notes = [
            len([n for n in notes if not n["is_rest"]])
            for notes, _ in all_line_data
        ]
        lyrics = extract_lyrics_with_claude(png_path, num_notes)

        out_dir = os.path.dirname(out_ly)
        os.makedirs(out_dir, exist_ok=True)

        notes_path = os.path.join(out_dir, "notes.ly")
        lyrics_path = os.path.join(out_dir, "lyrics.ly")

        with open(notes_path, "w") as f:
            f.write(build_notes_ly(all_lines, global_key))

        lyrics_content = build_lyrics_ly(lyrics)
        if lyrics_content:
            with open(lyrics_path, "w") as f:
                f.write(lyrics_content)

        if composer:
            with open(os.path.join(out_dir, "composer.txt"), "w") as f:
                f.write(composer)

        # Render SVG
        svg_path = os.path.join(out_dir, "1.svg")
        render_svg(notes_path, lyrics_path if lyrics_content else None,
                   svg_path, composer=composer)
        return (psalm_name, "OK", time.time() - t0)
    except Exception as e:
        return (psalm_name, f"FAILED: {e}", time.time() - t0)
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser(description="Batch process psalm images to LilyPond")
    parser.add_argument("-n", "--limit", type=int, help="Max number of psalms to process")
    parser.add_argument("-j", "--jobs", type=int, default=4, help="Parallel workers (default: 4)")
    args = parser.parse_args()

    # Find all psalm directories
    psalm_dirs = sorted(glob.glob(os.path.join(REPO_DIR, "photos", "psalm*")))
    print(f"Found {len(psalm_dirs)} psalm directories")

    # Build work items
    work = []
    for psalm_dir in psalm_dirs:
        psalm_name = os.path.basename(psalm_dir)
        out_dir = os.path.join(REPO_DIR, "lilypond", psalm_name)
        out_ly = os.path.join(out_dir, "1.ly")

        if os.path.exists(out_ly):
            print(f"  SKIP {psalm_name} (already exists)")
            continue

        png_path, needs_concat, parts = find_psalm_input(psalm_dir)

        if needs_concat and parts:
            # Concatenate parts and save in output directory
            os.makedirs(out_dir, exist_ok=True)
            concat_path = os.path.join(out_dir, "1.png")
            concatenate_images_vertically(parts, concat_path)
            work.append((concat_path, out_ly, concat_path))
        elif png_path:
            work.append((png_path, out_ly, None))
        else:
            print(f"  SKIP {psalm_name} (no 1.png or 1a.png)")

    if args.limit:
        work = work[:args.limit]

    print(f"Processing {len(work)} psalms with {args.jobs} workers")
    t_total = time.time()

    with ThreadPool(args.jobs) as pool:
        for psalm_name, status, elapsed in pool.imap_unordered(process_psalm, work):
            print(f"  {psalm_name}: {status} [{elapsed:.0f}s]")

    print(f"Done. Total: {time.time() - t_total:.0f}s")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
png2ly.py — Convert a scanned psalm image (Genevan Psalter style) to LilyPond.

Pipeline:
  1. Detect staff lines using OpenCV
  2. Crop each line with padding
  3. Run Audiveris OMR on each cropped line
  4. Parse MusicXML output
  5. Fix key signature (Audiveris loses it on continuation lines)
  6. Filter spurious articulations/dynamics
  7. Assemble into LilyPond with optional lyrics
  8. Optionally render PDF via lilypond

Usage:
  python3 png2ly.py input.png [--output output.ly] [--lyrics lyrics.txt]
         [--audiveris-dir ./audiveris] [--render]
"""

import argparse
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET

import cv2
import numpy as np


def run_claude(prompt, retries=3, timeout=120, model=None):
    """Run claude -p with retries on timeout, adding entropy each attempt."""
    import random
    for attempt in range(retries):
        if attempt > 0:
            fillers = [
                f" [attempt {attempt + 1}]",
                f" (please respond concisely, try #{attempt + 1})",
                f" — retry {attempt + 1}, be brief",
            ]
            varied_prompt = prompt + random.choice(fillers)
        else:
            varied_prompt = prompt
        try:
            cmd = ["claude", "-p", varied_prompt]
            if model:
                cmd.extend(["--model", model])
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                stdin=subprocess.DEVNULL,
                timeout=timeout,
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except subprocess.TimeoutExpired:
            if attempt < retries - 1:
                print(f" (retry {attempt + 2}/{retries})", end="", flush=True)
    return None


def extract_composer_with_claude(img_path):
    """Use claude CLI to extract the composer/tune attribution from the image."""
    prompt = (
        f"Read the image at {img_path} and then: "
        "Look at this hymn/psalm sheet music image. "
        "Extract ONLY the composer or tune attribution text "
        "(usually in the top right corner, e.g. 'Strasbourg, 1539 / Geneva, 1551'). "
        "Output nothing except the attribution text. No explanation."
    )
    text = run_claude(prompt)
    if not text:
        return None
    # Strip quotes if wrapped
    if text.startswith('"') and text.endswith('"'):
        text = text[1:-1]
    return text if text else None


def extract_lyrics_with_claude(img_path, num_notes_per_line):
    """Use claude CLI to extract lyrics from the image in LilyPond lyricmode format."""
    notes_info = ", ".join(str(n) for n in num_notes_per_line)
    prompt = (
        "Extract the lyrics from this hymn/psalm sheet music image. "
        "Output ONLY the lyrics in LilyPond \\lyricmode format. "
        "Rules:\n"
        "- Use ' -- ' (space-dash-dash-space) between syllables of the same word\n"
        "- Use spaces between separate words\n"
        "- Do not include verse numbers\n"
        "- Include all punctuation as it appears\n"
        f"- The music has {len(num_notes_per_line)} lines with this many notes per line: {notes_info}\n"
        "- Each syllable should align with one note. If a syllable spans multiple notes "
        "(melisma), use '_' for the extra notes after the syllable. \n"
        "- The number of notes in a line should match the number of syllables in a line\n"
        "- Output nothing except the lyricmode content (no \\lyricmode wrapper, no explanation)"
    )
    text = run_claude(f"Read the image at {img_path} and then: {prompt}")
    if not text:
        return None
    # Strip markdown code fences if present
    # Find the last code fence block if there is one
    if "```" in text:
        blocks = text.split("```")
        # Look for the content between the last pair of fences
        # blocks alternate: text, code, text, code, ...
        for i in range(len(blocks) - 1, 0, -1):
            candidate = blocks[i].strip()
            # Skip fence language markers
            for prefix in ["lilypond", "ly", "text"]:
                if candidate.startswith(prefix):
                    candidate = candidate[len(prefix):].strip()
            if candidate and "--" in candidate:
                return candidate
    # If no code fences, try to extract just the lyrics lines
    # Filter out lines that look like explanation rather than lyrics
    lines = []
    for line in text.splitlines():
        line = line.strip()
        # Skip empty lines, bullet points, explanations
        if not line or line.startswith("-") or line.startswith("*") or ":" in line[:20]:
            continue
        lines.append(line)
    return "\n".join(lines) if lines else text



def detect_staff_systems(img_path):
    """Detect staff line positions and return (top, bottom) row pairs."""
    img = cv2.imread(img_path, cv2.IMREAD_GRAYSCALE)
    if img is None:
        raise ValueError(f"Could not read image: {img_path}")

    h, w = img.shape

    # Horizontal blur to suppress vertical features (barlines, stems, text)
    # keeping only long horizontal features (staff lines)
    kernel_size = max(w // 8, 50)
    blurred = cv2.blur(img, (kernel_size, 1))
    row_darkness = np.sum(blurred < 160, axis=1)
    threshold = w * 0.3
    staff_rows = np.where(row_darkness > threshold)[0]

    if len(staff_rows) == 0:
        raise ValueError("No staff lines detected in image")

    gaps = np.diff(staff_rows)
    # Adaptive threshold: intra-staff gaps are small (~10-15px),
    # inter-system gaps are much larger (~40+px).
    # Use 2x the median gap as threshold — median will be an intra-staff gap.
    median_gap = np.median(gaps)
    gap_threshold = max(median_gap * 2, 20)
    breaks = np.where(gaps > gap_threshold)[0]
    groups = np.split(staff_rows, breaks + 1)

    systems = []
    for g in groups:
        if len(g) >= 3:  # filter noise
            systems.append((int(g[0]), int(g[-1])))

    return systems, h


MIN_INTERLINE_PX = 14  # Audiveris needs at least ~14px interline


def estimate_interline(systems):
    """Estimate interline spacing from detected staff systems."""
    if len(systems) == 0:
        return 0
    # Each system spans ~4 interlines (5 staff lines)
    spans = [bot - top for top, bot in systems]
    median_span = np.median(spans)
    return median_span / 4


def crop_lines(img_path, systems, img_height, output_dir, pad=40):
    """Crop each staff system, upscaling only if interline is too small."""
    img = cv2.imread(img_path)
    interline = estimate_interline(systems)
    scale = 1
    if interline < MIN_INTERLINE_PX:
        scale = int(np.ceil(MIN_INTERLINE_PX / interline))
        scale = max(2, min(scale, 4))

    paths = []
    for i, (top, bot) in enumerate(systems):
        y1 = max(0, top - pad)
        y2 = min(img_height, bot + pad)
        crop = img[y1:y2, :]
        if scale > 1:
            crop = cv2.resize(crop, None, fx=scale, fy=scale,
                              interpolation=cv2.INTER_NEAREST)
        path = os.path.join(output_dir, f"line{i + 1}.png")
        cv2.imwrite(path, crop)
        paths.append(path)
    return paths


AUDIVERIS_BIN = "/opt/audiveris/bin/Audiveris"


def run_audiveris(line_path, output_dir, audiveris_dir=None):
    """Run Audiveris on a single line image, return path to .mxl file."""
    cmd = [
        AUDIVERIS_BIN,
        "-batch",
        "-export",
        "-output", output_dir,
        line_path,
    ]
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=120
    )
    if result.returncode != 0:
        stderr = result.stdout + result.stderr
        if "Could not find file" in stderr:
            raise FileNotFoundError(f"Audiveris could not find: {line_path}")
        print(f"  Warning: Audiveris returned code {result.returncode}", file=sys.stderr)
        print(f"  {stderr[-200:]}", file=sys.stderr)

    basename = os.path.splitext(os.path.basename(line_path))[0]
    mxl_path = os.path.join(output_dir, f"{basename}.mxl")
    if not os.path.exists(mxl_path):
        raise FileNotFoundError(f"Audiveris did not produce: {mxl_path}")
    return mxl_path


def extract_mxl(mxl_path, output_dir):
    """Extract .mxl (zip) and return path to the inner .xml file."""
    import zipfile

    basename = os.path.splitext(os.path.basename(mxl_path))[0]
    extract_dir = os.path.join(output_dir, f"{basename}_xml")
    os.makedirs(extract_dir, exist_ok=True)
    with zipfile.ZipFile(mxl_path, "r") as z:
        z.extractall(extract_dir)
    xml_path = os.path.join(extract_dir, f"{basename}.xml")
    if not os.path.exists(xml_path):
        # Try to find any .xml file
        for f in os.listdir(extract_dir):
            if f.endswith(".xml") and f != "container.xml":
                xml_path = os.path.join(extract_dir, f)
                break
    return xml_path


def parse_musicxml(xml_path):
    """Parse MusicXML and return list of note dicts with pitch, duration, type, is_rest."""
    tree = ET.parse(xml_path)
    root = tree.getroot()

    notes = []
    key_fifths = None

    for measure in root.iter("measure"):
        # Check for key signature
        for attr in measure.iter("attributes"):
            key_elem = attr.find("key")
            if key_elem is not None:
                fifths_elem = key_elem.find("fifths")
                if fifths_elem is not None:
                    key_fifths = int(fifths_elem.text)

        for note_elem in measure.iter("note"):
            rest_elem = note_elem.find("rest")
            if rest_elem is not None:
                type_elem = note_elem.find("type")
                note_type = type_elem.text if type_elem is not None else "quarter"
                notes.append({"is_rest": True, "type": note_type})
                continue

            pitch_elem = note_elem.find("pitch")
            if pitch_elem is None:
                continue

            step = pitch_elem.find("step").text
            octave = int(pitch_elem.find("octave").text)
            alter_elem = pitch_elem.find("alter")
            alter = int(float(alter_elem.text)) if alter_elem is not None else 0

            type_elem = note_elem.find("type")
            note_type = type_elem.text if type_elem is not None else "quarter"

            notes.append({
                "is_rest": False,
                "step": step,
                "octave": octave,
                "alter": alter,
                "type": note_type,
            })

    return notes, key_fifths


# Sharp order: F C G D A E B
SHARP_ORDER = ["F", "C", "G", "D", "A", "E", "B"]
FLAT_ORDER = ["B", "E", "A", "D", "G", "C", "F"]


def apply_key_signature(notes, fifths):
    """Apply key signature sharps/flats to notes that don't have explicit accidentals."""
    if fifths is None or fifths == 0:
        return notes

    sharped = set()
    flatted = set()
    if fifths > 0:
        sharped = set(SHARP_ORDER[:fifths])
    elif fifths < 0:
        flatted = set(FLAT_ORDER[:abs(fifths)])

    for note in notes:
        if note["is_rest"]:
            continue
        step = note["step"]
        if note["alter"] == 0:
            if step in sharped:
                note["alter"] = 1
            elif step in flatted:
                note["alter"] = -1

    return notes


def note_to_lilypond(note):
    """Convert a single note dict to LilyPond string."""
    if note["is_rest"]:
        duration_map = {"whole": "1", "half": "2", "quarter": "4", "eighth": "8"}
        dur = duration_map.get(note["type"], "4")
        return f"r{dur}"

    step = note["step"].lower()
    alter = note["alter"]
    if alter == 1:
        step += "is"
    elif alter == -1:
        step += "es"

    duration_map = {
        "breve": "\\breve",
        "whole": "1",
        "half": "2",
        "quarter": "4",
        "eighth": "8",
    }
    dur = duration_map.get(note["type"], "4")

    return f"{step}{dur}"


def notes_to_lilypond_relative(notes):
    """Convert notes to LilyPond in \\relative mode, adding octave marks."""
    if not notes:
        return ""

    # Find first pitched note to set the reference
    first_pitched = None
    for n in notes:
        if not n["is_rest"]:
            first_pitched = n
            break

    if first_pitched is None:
        return " ".join(note_to_lilypond(n) for n in notes)

    result = []
    prev_octave = first_pitched["octave"]
    prev_step_idx = "CDEFGAB".index(first_pitched["step"])

    for note in notes:
        ly = note_to_lilypond(note)

        if not note["is_rest"]:
            cur_step_idx = "CDEFGAB".index(note["step"])
            cur_octave = note["octave"]

            # Calculate the expected octave in relative mode
            # LilyPond relative: choose the closest note
            interval = cur_step_idx - prev_step_idx
            if interval > 3:
                expected_octave = prev_octave - 1
            elif interval < -3:
                expected_octave = prev_octave + 1
            else:
                expected_octave = prev_octave

            diff = cur_octave - expected_octave
            if diff > 0:
                ly = ly.replace(note["step"].lower(), note["step"].lower() + "'" * diff, 1)
            elif diff < 0:
                step_with_alter = note["step"].lower()
                if note["alter"] == 1:
                    step_with_alter += "is"
                elif note["alter"] == -1:
                    step_with_alter += "es"
                ly = ly.replace(step_with_alter, step_with_alter + "," * abs(diff), 1)

            prev_octave = cur_octave
            prev_step_idx = cur_step_idx

        result.append(ly)

    return " ".join(result)


def detect_key_fifths(all_line_notes):
    """Detect key signature from the first line that has one."""
    for notes, fifths in all_line_notes:
        if fifths is not None:
            return fifths
    return 0


def fifths_to_lilypond_key(fifths):
    """Convert fifths value to LilyPond key string."""
    major_keys = {
        -7: "ces", -6: "ges", -5: "des", -4: "aes", -3: "ees",
        -2: "bes", -1: "f", 0: "c", 1: "g", 2: "d",
        3: "a", 4: "e", 5: "b", 6: "fis", 7: "cis",
    }
    key_name = major_keys.get(fifths, "c")
    return f"\\key {key_name} \\major"


def relative_header(fifths):
    """Determine the \\relative starting pitch from first note context."""
    # We'll use c'' as default (common for treble clef melodies)
    return "c''"


def build_notes_ly(all_lines, key_fifths):
    """Generate LilyPond melody definition."""
    key_str = fifths_to_lilypond_key(key_fifths)

    lines_ly = []
    for i, line_notes in enumerate(all_lines):
        ly = notes_to_lilypond_relative(line_notes)

        if i < len(all_lines) - 1:
            ly += ' \\break'
        else:
            ly += ' \\bar "|."'
        lines_ly.append(f"  % Line {i + 1}\n  {ly}\n")

    melody_block = "\n".join(lines_ly)

    return f"""melody = \\relative {relative_header(key_fifths)} {{
  \\clef treble
  {key_str}
  \\cadenzaOn
  \\omit Staff.TimeSignature

{melody_block}}}
"""


def sanitize_lyrics(lyrics):
    """Clean up lyrics for LilyPond lyricmode compatibility."""
    # Replace smart quotes with straight quotes
    lyrics = lyrics.replace("\u201c", '"').replace("\u201d", '"')
    lyrics = lyrics.replace("\u2018", "'").replace("\u2019", "'")
    # Remove invalid LilyPond commands that Claude sometimes inserts
    lyrics = re.sub(r'\\(left|right|textit|textbf|emph)\s*', '', lyrics)
    # Remove unicode escapes
    lyrics = re.sub(r'\\u[0-9a-fA-F]{4}', '', lyrics)
    return lyrics


def build_lyrics_ly(lyrics):
    """Generate LilyPond verse definition."""
    if not lyrics:
        return ""
    lyrics = sanitize_lyrics(lyrics)
    return f"""verse = \\lyricmode {{
  {lyrics}
}}
"""


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


def find_psalm_input(psalm_dir):
    """Determine the input PNG for a psalm directory.

    Returns (png_path, needs_concat, parts).
    """
    single = os.path.join(psalm_dir, "1.png")
    if os.path.exists(single):
        return single, False, []

    parts = sorted(glob.glob(os.path.join(psalm_dir, "1[a-z].png")))
    if parts:
        return None, True, parts

    return None, False, []


def process_psalm(png_path, out_dir, no_lyrics=False):
    """Run the full png2ly pipeline on a single image."""
    from render_ly import render_svg

    tmpdir = tempfile.mkdtemp(prefix="png2ly_")
    try:
        systems, img_height = detect_staff_systems(png_path)
        print(f"  Found {len(systems)} staff systems")
        line_paths = crop_lines(png_path, systems, img_height, tmpdir)

        all_line_data = []
        for i, line_path in enumerate(line_paths):
            try:
                mxl_path = run_audiveris(line_path, tmpdir)
                xml_path = extract_mxl(mxl_path, tmpdir)
                notes, key_fifths = parse_musicxml(xml_path)
                all_line_data.append((notes, key_fifths))
                print(f"  Line {i+1}: {len(notes)} notes, key={key_fifths}")
            except Exception as e:
                print(f"  Line {i+1}: FAILED {e}")
                all_line_data.append(([], None))

        global_key = detect_key_fifths(all_line_data)
        print(f"  Key: {global_key} fifths")
        all_lines = []
        for notes, _ in all_line_data:
            all_lines.append(apply_key_signature(notes, global_key))

        composer = extract_composer_with_claude(png_path)
        lyrics = None
        if not no_lyrics:
            num_notes = [
                len([n for n in notes if not n["is_rest"]])
                for notes, _ in all_line_data
            ]
            lyrics = extract_lyrics_with_claude(png_path, num_notes)

        os.makedirs(out_dir, exist_ok=True)

        notes_path = os.path.join(out_dir, "notes.ly")
        lyrics_path = os.path.join(out_dir, "lyrics_1.ly")

        with open(notes_path, "w") as f:
            f.write(build_notes_ly(all_lines, global_key))

        lyrics_content = build_lyrics_ly(lyrics)
        if lyrics_content:
            with open(lyrics_path, "w") as f:
                f.write(lyrics_content)

        if composer:
            with open(os.path.join(out_dir, "composer.txt"), "w") as f:
                f.write(composer)

        svg_path = os.path.join(out_dir, "1.svg")
        render_svg(notes_path, lyrics_path if lyrics_content else None,
                   svg_path, composer=composer)
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def _process_psalm_worker(args):
    """Worker wrapper for process_psalm. Returns (psalm_name, status, elapsed)."""
    png_path, out_dir, psalm_name = args
    t0 = time.time()
    try:
        process_psalm(png_path, out_dir)
        return (psalm_name, "OK", time.time() - t0)
    except Exception as e:
        return (psalm_name, f"FAILED: {e}", time.time() - t0)


def main():
    parser = argparse.ArgumentParser(
        description="Convert scanned psalm images to LilyPond notation using Audiveris OMR"
    )
    parser.add_argument("input", nargs="?", help="Path to input PNG image (single mode)")
    parser.add_argument("-o", "--output", help="Output directory (single mode)")
    parser.add_argument("--all", action="store_true", help="Batch process all psalms")
    parser.add_argument("-n", "--limit", type=int, help="Max number of psalms to process (batch mode)")
    parser.add_argument("-j", "--jobs", type=int, default=4, help="Parallel workers for batch mode (default: 4)")
    parser.add_argument("--psalm", help="Process only this psalm (e.g. psalm6)")
    parser.add_argument("--no-lyrics", action="store_true", help="Skip lyrics extraction")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_dir = os.path.dirname(script_dir)

    if args.all or args.psalm:
        # Batch mode
        from multiprocessing.pool import ThreadPool

        photos_dir = os.path.join(repo_dir, "photos")
        lilypond_dir = os.path.join(repo_dir, "lilypond")

        if args.psalm:
            psalm_dirs = [os.path.join(photos_dir, args.psalm)]
        else:
            psalm_dirs = sorted(glob.glob(os.path.join(photos_dir, "psalm*")))

        print(f"Found {len(psalm_dirs)} psalm directories")

        work = []
        for psalm_dir in psalm_dirs:
            psalm_name = os.path.basename(psalm_dir)
            out_dir = os.path.join(lilypond_dir, psalm_name)
            notes_path = os.path.join(out_dir, "notes.ly")

            if os.path.exists(notes_path):
                print(f"  SKIP {psalm_name} (already exists)")
                continue

            png_path, needs_concat, parts = find_psalm_input(psalm_dir)

            if needs_concat and parts:
                os.makedirs(out_dir, exist_ok=True)
                concat_path = os.path.join(out_dir, "1.png")
                concatenate_images_vertically(parts, concat_path)
                work.append((concat_path, out_dir, psalm_name))
            elif png_path:
                work.append((png_path, out_dir, psalm_name))
            else:
                print(f"  SKIP {psalm_name} (no 1.png or 1a.png)")

        if args.limit:
            work = work[:args.limit]

        print(f"Processing {len(work)} psalms with {args.jobs} workers")
        t_total = time.time()

        with ThreadPool(args.jobs) as pool:
            for psalm_name, status, elapsed in pool.imap_unordered(_process_psalm_worker, work):
                print(f"  {psalm_name}: {status} [{elapsed:.0f}s]")

        print(f"Done. Total: {time.time() - t_total:.0f}s")

    elif args.input:
        # Single image mode
        input_path = os.path.abspath(args.input)
        if not os.path.exists(input_path):
            print(f"Error: {input_path} not found", file=sys.stderr)
            sys.exit(1)

        if args.output:
            out_dir = args.output
        else:
            out_dir = os.path.splitext(input_path)[0]

        t0 = time.time()
        process_psalm(input_path, out_dir, no_lyrics=args.no_lyrics)
        print(f"Done. Total: {time.time() - t0:.1f}s")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()

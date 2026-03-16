#!/usr/bin/env python3
"""Render LilyPond SVG from separate notes and lyrics files."""

import argparse
import glob
import os
import re
import subprocess
from multiprocessing.pool import ThreadPool


def render_svg(notes_path, lyrics_path, output_svg, composer=None):
    """Combine notes and lyrics files with header/footer and render to SVG.

    Args:
        notes_path: Path to .ly file containing melody definition
        lyrics_path: Path to .ly file containing verse definition (or None)
        output_svg: Path for output SVG file
        composer: Optional composer attribution string
    """
    header_items = []
    if composer:
        header_items.append(f'  composer = "{composer}"')
    header_items.append("  tagline = ##f")
    header_block = "\\header {\n" + "\n".join(header_items) + "\n}"

    # Read notes and add visual adjustments
    with open(notes_path) as f:
        notes_content = f.read()

    # Hide clef after first line
    notes_content = notes_content.replace(
        "\\break", "\\break\n  \\omit Staff.Clef", 1
    )

    # Add hidden rests for line alignment
    # Check if any line starts/ends with a rest
    lines = notes_content.split("\\break")
    any_start_rest = any(re.search(r'^\s*r[0-9]', l.split("\n")[-1].strip() if "\n" in l else l.strip()) for l in lines)
    any_end_rest = any(re.search(r'r[0-9]\s*$', l.strip().rstrip("\\bar \"|.\"")) for l in lines)
    if not any_end_rest:
        any_end_rest = any("r2" in l.split("\\break")[0][-10:] if "\\break" not in l else "r2" in l[-10:] for l in lines)

    # Simpler: scan for r4 at line starts and r2 at line ends
    note_lines = [l.strip() for l in notes_content.split("\\break")]
    # Check start rests (look for r4 or r2 as first token after % Line comment)
    def line_content(l):
        """Extract note content, skipping comments and clef overrides."""
        result = []
        for part in l.split("\n"):
            part = part.strip()
            if part.startswith("%") or part.startswith("\\omit") or not part:
                continue
            result.append(part)
        return " ".join(result)

    line_contents = [line_content(l) for l in note_lines if line_content(l)]
    any_start_rest = any(lc.startswith("r") for lc in line_contents)
    any_end_rest = any(re.search(r'r[12]\s*$', lc) for lc in line_contents)

    if any_start_rest or any_end_rest:
        new_lines = []
        for segment in notes_content.split("\n"):
            stripped = segment.strip()
            # Skip non-note lines
            if stripped.startswith("%") or stripped.startswith("\\") or not stripped or stripped == "}":
                new_lines.append(segment)
                continue
            # Check if this line has notes
            if not re.search(r'[a-g]', stripped):
                new_lines.append(segment)
                continue
            # Add hidden rest at start if needed
            if any_start_rest and not stripped.startswith("r"):
                segment = segment.replace(stripped, "\\once \\hide Rest r4 " + stripped, 1)
                stripped = "\\once \\hide Rest r4 " + stripped
            # Add hidden rest at end if needed
            if any_end_rest and not re.search(r'r[12]\s*(\\break|\\bar)', stripped):
                segment = re.sub(r'(\s*\\break|\s*\\bar)', r' \\once \\hide Rest r2\1', segment, count=1)
            new_lines.append(segment)
        notes_content = "\n".join(new_lines)

    # Read lyrics if provided, sanitize for LilyPond
    lyrics_content = ""
    if lyrics_path and os.path.exists(lyrics_path):
        with open(lyrics_path) as f:
            lyrics_content = f.read()
        # Sanitize
        lyrics_content = lyrics_content.replace("\u201c", '"').replace("\u201d", '"')
        lyrics_content = lyrics_content.replace("\u2018", "'").replace("\u2019", "'")
        lyrics_content = re.sub(r'\\(left|right|textit|textbf|emph)\s*', '', lyrics_content)
        lyrics_content = re.sub(r'\\u[0-9a-fA-F]{4}', '', lyrics_content)

    lyrics_score = ""
    if lyrics_content.strip():
        lyrics_score = '    \\new Lyrics \\lyricsto "melody" { \\verse }'

    combined = f"""\\version "2.24.0"

\\paper {{
  line-width = 13\\cm
  left-margin = 0\\cm
  right-margin = 0\\cm
}}

{header_block}

{notes_content}

{lyrics_content}

\\score {{
  <<
    \\new Voice = "melody" {{ \\melody }}
{lyrics_score}
  >>
  \\layout {{
    indent = 0
    \\context {{
      \\Lyrics
      \\override LyricText.self-alignment-X = #LEFT
    }}
  }}
}}
"""

    # Write combined .ly to a temp file next to the output
    out_dir = os.path.dirname(os.path.abspath(output_svg))
    os.makedirs(out_dir, exist_ok=True)
    svg_base = os.path.splitext(os.path.basename(output_svg))[0]
    combined_ly = os.path.join(out_dir, f"_combined_{svg_base}.ly")
    with open(combined_ly, "w") as f:
        f.write(combined)

    # Render
    result = subprocess.run(
        ["lilypond", "-dbackend=svg", "-o",
         os.path.splitext(os.path.abspath(output_svg))[0],
         combined_ly],
        capture_output=True,
        text=True,
        cwd=out_dir,
    )

    # Clean up temp file
    if os.path.exists(combined_ly):
        os.remove(combined_ly)

    return result.returncode == 0


def _render_one(args):
    """Render a single lyrics file. Returns (label, success)."""
    lyrics_path, = args
    psalm_dir = os.path.dirname(lyrics_path)
    psalm_name = os.path.basename(psalm_dir)
    notes_path = os.path.join(psalm_dir, "notes.ly")

    if not os.path.exists(notes_path):
        return (f"{psalm_name}", "SKIP")

    basename = os.path.basename(lyrics_path)
    verse_num = basename.replace("lyrics_", "").replace(".ly", "")
    svg_path = os.path.join(psalm_dir, f"{verse_num}.svg")

    composer = None
    composer_path = os.path.join(psalm_dir, "composer.txt")
    if os.path.exists(composer_path):
        with open(composer_path) as f:
            composer = f.read().strip()

    ok = render_svg(notes_path, lyrics_path, svg_path, composer=composer)
    return (f"{psalm_name} v{verse_num}", "OK" if ok else "FAILED")


def main():
    """Render all psalms in the lilypond directory."""
    parser = argparse.ArgumentParser(description="Render all psalm SVGs")
    parser.add_argument("-j", "--jobs", type=int, default=os.cpu_count(), help="Parallel workers (default: nproc)")
    parser.add_argument("--psalm", help="Process only this psalm (e.g. psalm102)")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_dir = os.path.dirname(script_dir)
    lilypond_dir = os.path.join(repo_dir, "lilypond")

    if args.psalm:
        pattern = os.path.join(lilypond_dir, args.psalm, "lyrics_*.ly")
    else:
        pattern = os.path.join(lilypond_dir, "psalm*", "lyrics_*.ly")
    lyrics_files = sorted(glob.glob(pattern))
    print(f"Found {len(lyrics_files)} lyrics files")

    work = [(lf,) for lf in lyrics_files]
    ok = failed = skipped = 0

    with ThreadPool(args.jobs) as pool:
        for label, status in pool.imap_unordered(_render_one, work):
            print(f"  {status} {label}")
            if status == "OK":
                ok += 1
            elif status == "FAILED":
                failed += 1
            else:
                skipped += 1

    print(f"Done. {ok} ok, {failed} failed, {skipped} skipped")


if __name__ == "__main__":
    main()

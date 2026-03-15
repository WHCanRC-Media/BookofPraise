#!/usr/bin/env python3
"""Render LilyPond SVG from separate notes and lyrics files."""

import glob
import os
import subprocess


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

    # Read notes
    with open(notes_path) as f:
        notes_content = f.read()

    # Read lyrics if provided
    lyrics_content = ""
    if lyrics_path and os.path.exists(lyrics_path):
        with open(lyrics_path) as f:
            lyrics_content = f.read()

    lyrics_score = ""
    if lyrics_content.strip():
        lyrics_score = '    \\new Lyrics \\lyricsto "melody" { \\verse }'

    combined = f"""\\version "2.24.0"

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
    line-width = 14\\cm
    ragged-right = ##f
  }}
  \\midi {{ \\tempo 2 = 72 }}
}}
"""

    # Write combined .ly to a temp file next to the output
    out_dir = os.path.dirname(os.path.abspath(output_svg))
    os.makedirs(out_dir, exist_ok=True)
    combined_ly = os.path.join(out_dir, "_combined.ly")
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


def main():
    """Render all psalms in the lilypond directory."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_dir = os.path.dirname(script_dir)
    lilypond_dir = os.path.join(repo_dir, "lilypond")

    # Find all lyrics files, derive psalm dirs and verse numbers
    lyrics_files = sorted(glob.glob(os.path.join(lilypond_dir, "psalm*", "lyrics_*.ly")))
    print(f"Found {len(lyrics_files)} lyrics files")

    ok = 0
    failed = 0
    skipped = 0

    for lyrics_path in lyrics_files:
        psalm_dir = os.path.dirname(lyrics_path)
        psalm_name = os.path.basename(psalm_dir)
        notes_path = os.path.join(psalm_dir, "notes.ly")

        if not os.path.exists(notes_path):
            print(f"  SKIP {psalm_name} (no notes.ly)")
            skipped += 1
            continue

        # Extract verse number from lyrics_N.ly
        basename = os.path.basename(lyrics_path)
        verse_num = basename.replace("lyrics_", "").replace(".ly", "")
        svg_path = os.path.join(psalm_dir, f"{verse_num}.svg")

        composer = None
        composer_path = os.path.join(psalm_dir, "composer.txt")
        if os.path.exists(composer_path):
            with open(composer_path) as f:
                composer = f.read().strip()

        if render_svg(notes_path, lyrics_path, svg_path, composer=composer):
            print(f"  OK {psalm_name} v{verse_num}")
            ok += 1
        else:
            print(f"  FAILED {psalm_name} v{verse_num}")
            failed += 1

    print(f"Done. {ok} ok, {failed} failed, {skipped} skipped")


if __name__ == "__main__":
    main()

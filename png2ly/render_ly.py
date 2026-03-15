#!/usr/bin/env python3
"""Render LilyPond SVG from separate notes and lyrics files."""

import os
import subprocess
import tempfile


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

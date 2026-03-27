#!/usr/bin/env python3
"""Render LilyPond SVG from separate notes and lyrics files."""

import argparse
import glob
import os
import re
import subprocess
from multiprocessing.pool import ThreadPool


def _extract_line_contents(notes_content):
    """Extract note content per line, skipping comments and overrides."""
    result = []
    for segment in notes_content.split("\\break"):
        parts = []
        for line in segment.split("\n"):
            line = line.strip()
            if line.startswith("%") or line.startswith("\\omit") or not line:
                continue
            parts.append(line)
        content = " ".join(parts)
        if content:
            result.append(content)
    return result


def modify_notes(notes_content, force_combine=False):
    """Apply visual adjustments to notes content for rendering.

    - If force_combine, merge line pairs by removing odd-numbered \\break markers
    - Inject accidentalStyle forget if no barlines
    - Hide clef after first line
    - Add hidden rests at start/end of lines for alignment
    """
    # Combine line pairs by removing odd-numbered \break markers
    if force_combine:
        parts = notes_content.split("\\break")
        combined = parts[0]
        for i, part in enumerate(parts[1:], 1):
            if i % 2 == 1:
                # Odd-numbered break: skip it (merge lines)
                combined += part
            else:
                combined += "\\break" + part
        notes_content = combined

    # If no standalone | barline separators, inject accidentalStyle forget
    has_barlines = any(
        "|" in tok
        for line in notes_content.splitlines()
        if not line.strip().startswith("%")
        for tok in line.split()
        if tok == "|"
    )
    if not has_barlines:
        notes_content = notes_content.replace(
            "\\cadenzaOn", "\\cadenzaOn\n  \\accidentalStyle forget", 1
        )

    # Hide clef after first line
    notes_content = notes_content.replace(
        "\\break", "\\break\n  \\omit Staff.Clef", 1
    )

    note_line_idx = 0
    new_lines = []
    for segment in notes_content.split("\n"):
        stripped = segment.strip()
        # Skip non-note lines (comments, overrides, braces, melody= header, empty)
        if (stripped.startswith("%") or stripped.startswith("\\") or
                not stripped or stripped == "}" or "melody" in stripped or
                "=" in stripped):
            new_lines.append(segment)
            continue
        # Check if this line has actual note patterns (letter with optional octave marks and digit)
        if not re.search(r"[a-g](is|es)?[',]*[0-9]", stripped):
            new_lines.append(segment)
            continue
        is_odd = note_line_idx % 2 == 0  # 0-indexed: 0,2,4 are odd original lines
        note_line_idx += 1
        # Add hidden rest at start (only for odd lines when combining, all lines otherwise)
        if not stripped.startswith("r") and (not force_combine or is_odd):
            segment = segment.replace(stripped, "\\once \\hide Rest r4 " + stripped, 1)
            stripped = "\\once \\hide Rest r4 " + stripped
        # Add hidden rest at end if needed (match any rest duration, not just r1/r2)
        if not re.search(r'r[0-9]+\.?\s*(\\break|\\bar)', stripped):
            segment = re.sub(r'(\s*\\break|\s*\\bar)', r' \\once \\hide Rest r2\1', segment, count=1)
        new_lines.append(segment)
    notes_content = "\n".join(new_lines)

    return notes_content


COMBINE_LINES_THRESHOLD = 7

NOTE_NAME_TO_PC = {'c': 0, 'd': 1, 'e': 2, 'f': 3, 'g': 4, 'a': 5, 'b': 6}
PC_TO_NOTE_NAME = {v: k for k, v in NOTE_NAME_TO_PC.items()}


def _parse_abs_pitch(s):
    """Parse a LilyPond pitch like c', gis, bes'' into (pc, octave)."""
    if not s or s[0] not in NOTE_NAME_TO_PC:
        return None
    pc = NOTE_NAME_TO_PC[s[0]]
    rest = s[1:]
    # Strip accidentals
    for acc in ("isis", "eses", "is", "es"):
        if rest.startswith(acc):
            rest = rest[len(acc):]
            break
    ups = rest.count("'")
    downs = rest.count(",")
    return (pc, ups - downs)


def _resolve_relative(prev, token):
    """Resolve a relative note token to absolute pitch given previous pitch."""
    if not token or token[0] not in NOTE_NAME_TO_PC:
        return None
    pc = NOTE_NAME_TO_PC[token[0]]
    rest = token[1:]
    for acc in ("isis", "eses", "is", "es"):
        if rest.startswith(acc):
            rest = rest[len(acc):]
            break
    ups = rest.count("'")
    downs = rest.count(",")
    prev_pc, prev_oct = prev
    diff = pc - prev_pc
    if diff > 3:
        octave = prev_oct - 1
    elif diff < -3:
        octave = prev_oct + 1
    else:
        octave = prev_oct
    return (pc, octave + ups - downs)


def _pitch_to_ly(pitch):
    """Format (pc, octave) as a LilyPond pitch string."""
    pc, octave = pitch
    name = PC_TO_NOTE_NAME[pc % 7]
    if octave > 0:
        marks = "'" * octave
    elif octave < 0:
        marks = "," * (-octave)
    else:
        marks = ""
    return f"{name}{marks}"


def _track_pitch(content, start):
    """Track absolute pitch through LilyPond notes in relative mode."""
    # Strip commands, quoted strings, and comments
    cleaned = re.sub(r"\\[a-zA-Z]+", " ", content)
    cleaned = re.sub(r'"[^"]*"', " ", cleaned)
    cleaned = re.sub(r"%.*", " ", cleaned)
    current = start
    for m in re.finditer(r"[a-g](is|es|isis|eses)?[',]*", cleaned):
        resolved = _resolve_relative(current, m.group())
        if resolved:
            current = resolved
    return current


def _effective_n_parts(split_style, break_count):
    """Determine number of slide parts based on split style and break count."""
    if split_style in ("default", "multi slide"):
        total_lines = break_count + 1
        return (total_lines + COMBINE_LINES_THRESHOLD - 1) // COMBINE_LINES_THRESHOLD
    return 1  # single slide, combine lines


def _count_note_lines(raw_notes):
    """Count note lines (segments separated by \\break or \\bar)."""
    return raw_notes.count("\\break") + raw_notes.count("\\bar")


def _split_notes(raw_notes, n_parts):
    """Split notes into n_parts at \\break boundaries, tracking relative pitch."""
    if n_parts <= 1:
        return [raw_notes]

    preamble_lines = []
    body = ""
    in_body = False
    relative_pitch = "c'"

    for line in raw_notes.splitlines():
        if not in_body:
            preamble_lines.append(line)
            m = re.search(r"\\relative\s*([a-g](is|es)?[',]*)", line)
            if m:
                relative_pitch = m.group(1)
            if '{' in line:
                in_body = True
        else:
            if line.strip() == "}":
                continue
            stripped = line.strip()
            if stripped and (stripped.startswith("\\clef") or stripped.startswith("\\key")
                    or stripped.startswith("\\cadenzaOn") or stripped.startswith("\\omit")):
                preamble_lines.append(line)
                continue
            body += line + "\n"

    # Split body into note lines at \break / \bar boundaries
    note_lines = []
    current_line = ""
    for line in body.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("%"):
            if current_line:
                current_line += "\n"
            current_line += line
            continue
        if current_line:
            current_line += "\n"
        current_line += line
        if "\\break" in stripped or "\\bar" in stripped:
            note_lines.append(current_line)
            current_line = ""
    if current_line.strip():
        note_lines.append(current_line)

    total_lines = len(note_lines)
    lines_per_part = (total_lines + n_parts - 1) // n_parts

    # Build setup lines (commands from preamble, excluding \relative and melody=)
    setup_lines = []
    for line in preamble_lines:
        if "\\relative" not in line and "melody" not in line:
            stripped = line.strip()
            if stripped and stripped != "{":
                setup_lines.append(stripped)

    start_pitch = _parse_abs_pitch(relative_pitch)
    if not start_pitch:
        start_pitch = (0, 1)  # c'
    current_pitch = start_pitch
    parts = []

    for part_idx in range(n_parts):
        start = part_idx * lines_per_part
        end = min((part_idx + 1) * lines_per_part, total_lines)
        if start >= total_lines:
            break

        pitch_str = relative_pitch if part_idx == 0 else _pitch_to_ly(current_pitch)
        setup = "\n  ".join(setup_lines)
        part_body = ""

        for i, line_idx in enumerate(range(start, end)):
            line = note_lines[line_idx]
            if i == end - start - 1:
                # Last line of this part
                if part_idx < n_parts - 1:
                    line = line.replace("\\break", "")
            part_body += line + "\n"
            current_pitch = _track_pitch(line, current_pitch)

        part = f"melody = \\relative {pitch_str} {{\n  {setup}\n\n{part_body}}}\n"
        parts.append(part)

    return parts


def _split_lyrics(raw_lyrics, n_parts, total_note_lines):
    """Split lyrics into n_parts groups matching the note line split."""
    if n_parts <= 1:
        return [raw_lyrics]

    preamble = ""
    lyric_lines = []
    in_body = False

    for line in raw_lyrics.splitlines():
        if not in_body:
            if "\\lyricmode" in line or "{" in line:
                preamble = line.split("{")[0] if "{" in line else "verse = \\lyricmode"
                after = line.split("{", 1)[1] if "{" in line else ""
                content = after.strip().rstrip("}").strip()
                if content:
                    lyric_lines.append(content)
                in_body = True
        else:
            stripped = line.strip()
            if stripped == "}":
                break
            if stripped:
                lyric_lines.append(stripped)

    lines_per_part = (total_note_lines + n_parts - 1) // n_parts
    parts = []

    for part_idx in range(n_parts):
        start = part_idx * lines_per_part
        end = min((part_idx + 1) * lines_per_part, len(lyric_lines))
        if start >= len(lyric_lines):
            parts.append(f"{preamble} {{\n}}\n")
            continue
        body = "\n".join(f"  {l}" for l in lyric_lines[start:end])
        parts.append(f"{preamble} {{\n{body}\n}}\n")

    return parts


def _count_pitched_notes(notes_content):
    """Count non-rest notes in a melody definition."""
    # Strip comments
    lines = []
    for line in notes_content.splitlines():
        line = line.split("%")[0]
        lines.append(line)
    content = " ".join(lines)
    # Match note tokens (letter + optional accidental + octave marks + duration)
    # but exclude rests (r followed by duration)
    notes = re.findall(r"[a-g](is|es)?[',]*\d", content)
    return len(notes)


def _count_syllables(lyrics_content):
    """Count syllables in a lyricmode block.

    Each whitespace-separated token is a syllable, except '--' (hyphen
    separator) and LilyPond commands.  '_' (melisma extender) counts as
    consuming a note.
    """
    # Strip the \lyricmode { ... } wrapper
    content = lyrics_content
    content = re.sub(r"\\lyricmode\s*\{", "", content)
    content = re.sub(r"verse\s*=\s*", "", content)
    # Remove trailing }
    content = re.sub(r"\}\s*$", "", content)
    # Collapse quoted strings into single tokens (but not escaped quotes \")
    content = re.sub(r'(?<!\\)"[^"]*(?<!\\)"', 'QUOTED', content)
    tokens = content.split()
    count = 0
    for tok in tokens:
        if tok == "--":
            continue
        if tok.startswith("\\"):
            continue
        count += 1
    return count


def render_svg_from_content(notes_content, lyrics_content, output_svg, composer=None):
    """Render pre-split notes and lyrics content to SVG.

    Used for multi-part rendering where notes/lyrics have already been split.
    """
    header_items = []
    if composer:
        header_items.append(f'  composer = "{composer}"')
    header_items.append("  tagline = ##f")
    header_block = "\\header {\n" + "\n".join(header_items) + "\n}"

    modified = modify_notes(notes_content, force_combine=False)
    paper_width_mm = max(
        len(re.findall(r"[a-gr](is|es)?[',]*\d", line))
        for line in _extract_line_contents(notes_content)
    ) * 9 + 20 if _extract_line_contents(notes_content) else 8 * 9 + 20

    lyrics_content = lyrics_content.replace('\\"', '\u201c')
    lyrics_score = ""
    if lyrics_content.strip():
        lyrics_score = '    \\new Lyrics \\lyricsto "melody" { \\verse }'

    combined = f"""\\version "2.24.0"

\\paper {{
  paper-width = {paper_width_mm}\\mm
  line-width = {paper_width_mm}\\mm
  left-margin = 0\\cm
  right-margin = 0\\cm
}}

{header_block}

{modified}

{lyrics_content}

\\score {{
  <<
    \\new Voice = "melody" {{ \\melody }}
{lyrics_score}
  >>
  \\layout {{
    indent = 0
    \\context {{
      \\Score
      \\override SpacingSpanner.uniform-stretching = ##t
      \\override SpacingSpanner.strict-note-spacing = ##t
    }}
    \\context {{
      \\Lyrics
      \\override LyricText.self-alignment-X = #LEFT
    }}
  }}
}}
"""

    out_dir = os.path.dirname(os.path.abspath(output_svg))
    os.makedirs(out_dir, exist_ok=True)
    svg_base = os.path.splitext(os.path.basename(output_svg))[0]
    combined_ly = os.path.join(out_dir, f"_combined_{svg_base}.ly")
    with open(combined_ly, "w") as f:
        f.write(combined)

    result = subprocess.run(
        ["lilypond", "-dbackend=svg", "-dcrop", "-o",
         os.path.splitext(os.path.abspath(output_svg))[0],
         combined_ly],
        capture_output=True,
        text=True,
        cwd=out_dir,
    )

    if result.returncode == 0:
        abs_svg = os.path.abspath(output_svg)
        cropped_svg = os.path.splitext(abs_svg)[0] + ".cropped.svg"
        if os.path.exists(cropped_svg):
            os.replace(cropped_svg, abs_svg)
        if os.path.exists(combined_ly):
            os.remove(combined_ly)

    return result.returncode == 0


def render_svg(notes_path, lyrics_path, output_svg, composer=None, split_style="default"):
    """Combine notes and lyrics files with header/footer and render to SVG.

    Args:
        notes_path: Path to .ly file containing melody definition
        lyrics_path: Path to .ly file containing verse definition (or None)
        output_svg: Path for output SVG file
        composer: Optional composer attribution string
        split_style: One of "default", "single slide", "multi slide", "combine lines"
    """
    header_items = []
    if composer:
        header_items.append(f'  composer = "{composer}"')
    header_items.append("  tagline = ##f")
    header_block = "\\header {\n" + "\n".join(header_items) + "\n}"

    with open(notes_path) as f:
        notes_content = f.read()

    # Calculate width from max notes per line
    line_contents = _extract_line_contents(notes_content)
    max_notes = max(
        len(re.findall(r"[a-gr](is|es)?[',]*\d", line))
        for line in line_contents
    ) if line_contents else 8
    paper_width_mm = max_notes * 9 + 20

    force_combine = split_style == "combine lines"
    notes_content = modify_notes(notes_content, force_combine=force_combine)

    # Read lyrics if provided, sanitize for LilyPond
    lyrics_content = ""
    if lyrics_path and os.path.exists(lyrics_path):
        with open(lyrics_path) as f:
            lyrics_content = f.read()
        lyrics_content = lyrics_content.replace('\\"', '\u201c')

    lyrics_score = ""
    if lyrics_content.strip():
        note_count = _count_pitched_notes(notes_content)
        syllable_count = _count_syllables(lyrics_content)
        if note_count != syllable_count:
            import warnings
            warnings.warn(
                f"{notes_path}: note/syllable mismatch: "
                f"{note_count} notes vs {syllable_count} syllables"
            )
        lyrics_score = '    \\new Lyrics \\lyricsto "melody" { \\verse }'

    combined = f"""\\version "2.24.0"

\\paper {{
  paper-width = {paper_width_mm}\\mm
  line-width = {paper_width_mm}\\mm
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
      \\Score
      \\override SpacingSpanner.uniform-stretching = ##t
      \\override SpacingSpanner.strict-note-spacing = ##t
    }}
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
        ["lilypond", "-dbackend=svg", "-dcrop", "-o",
         os.path.splitext(os.path.abspath(output_svg))[0],
         combined_ly],
        capture_output=True,
        text=True,
        cwd=out_dir,
    )

    if result.returncode == 0:
        # Replace full-page SVG with cropped version
        abs_svg = os.path.abspath(output_svg)
        cropped_svg = os.path.splitext(abs_svg)[0] + ".cropped.svg"
        if os.path.exists(cropped_svg):
            os.replace(cropped_svg, abs_svg)
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

    composer = None
    split_style = "default"
    yaml_path = os.path.join(psalm_dir, "song.yaml")
    if os.path.exists(yaml_path):
        import yaml
        with open(yaml_path) as f:
            meta = yaml.safe_load(f) or {}
        composer = meta.get("composer")
        split_style = meta.get("split_style", "default")

    with open(notes_path) as f:
        raw_notes = f.read()

    break_count = raw_notes.count("\\break")
    n_parts = _effective_n_parts(split_style, break_count)

    if n_parts > 1:
        with open(lyrics_path) as f:
            raw_lyrics = f.read()
        raw_lyrics = raw_lyrics.replace('\\"', '\u201c')
        total_note_lines = _count_note_lines(raw_notes)
        note_parts = _split_notes(raw_notes, n_parts)
        lyric_parts = _split_lyrics(raw_lyrics, n_parts, total_note_lines)

        all_ok = True
        for part_idx, (notes_part, lyrics_part) in enumerate(zip(note_parts, lyric_parts)):
            svg_path = os.path.join(psalm_dir, f"{verse_num}_p{part_idx}.svg")
            comp = composer if part_idx == 0 else None
            ok = render_svg_from_content(notes_part, lyrics_part, svg_path, composer=comp)
            if not ok:
                all_ok = False
        label = f"{psalm_name} v{verse_num} ({len(note_parts)} parts)"
        return (label, "OK" if all_ok else "FAILED")
    else:
        svg_path = os.path.join(psalm_dir, f"{verse_num}.svg")
        ok = render_svg(notes_path, lyrics_path, svg_path, composer=composer, split_style=split_style)
        return (f"{psalm_name} v{verse_num}", "OK" if ok else "FAILED")


def main():
    """Render all psalms in the lilypond directory."""
    parser = argparse.ArgumentParser(description="Render all psalm SVGs")
    parser.add_argument("-j", "--jobs", type=int, default=os.cpu_count(), help="Parallel workers (default: nproc)")
    parser.add_argument("--psalm", help="Process only this psalm (e.g. psalm102)")
    parser.add_argument("--check", action="store_true", help="Check note/syllable counts without rendering")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_dir = os.path.dirname(script_dir)
    lilypond_dir = os.path.join(repo_dir, "lilypond")

    if args.psalm:
        pattern = os.path.join(lilypond_dir, args.psalm, "lyrics_*.ly")
    else:
        pattern = os.path.join(lilypond_dir, "*", "lyrics_*.ly")
    lyrics_files = sorted(glob.glob(pattern))

    if args.check:
        ok = failed = 0
        for lyrics_path in lyrics_files:
            song_dir = os.path.dirname(lyrics_path)
            song_name = os.path.basename(song_dir)
            notes_path = os.path.join(song_dir, "notes.ly")
            if not os.path.exists(notes_path):
                continue
            with open(notes_path) as f:
                note_count = _count_pitched_notes(f.read())
            with open(lyrics_path) as f:
                syllable_count = _count_syllables(f.read())
            basename = os.path.basename(lyrics_path).replace("lyrics_", "").replace(".ly", "")
            label = f"{song_name} v{basename}"
            if note_count == syllable_count:
                ok += 1
            else:
                failed += 1
                print(f"  MISMATCH {label}: {note_count} notes vs {syllable_count} syllables")
        print(f"Checked {ok + failed}, {failed} mismatches")
        return

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

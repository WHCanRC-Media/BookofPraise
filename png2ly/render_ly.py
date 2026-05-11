#!/usr/bin/env python3
"""Render LilyPond SVG from separate notes and lyrics files."""

import argparse
import glob
import os
import re
import shutil
import subprocess
import sys
from multiprocessing.pool import ThreadPool


def _lilypond_bin():
    """Resolve the lilypond binary, preferring the bundled one in the bop cache."""
    if sys.platform == "win32":
        base = os.environ.get("LOCALAPPDATA") or "C:\\Temp"
        bin_name = "lilypond.exe"
    else:
        base = os.environ.get("XDG_CACHE_HOME") or os.path.join(
            os.environ.get("HOME", "/tmp"), ".cache"
        )
        bin_name = "lilypond"
    bundled = os.path.join(base, "bop", "lilypond-bin", "bin", bin_name)
    if os.path.exists(bundled):
        return bundled
    return shutil.which(bin_name) or bin_name


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


_NOTE_RE = r"[a-g](?:isis|eses|is|es)?[',]*"


def _beam_slurred_eighths(notes_content):
    """Beam pairs of slurred eighth notes.

    Only matches slurs that contain exactly two plain eighth notes
    (e.g. `a8( b8)` -> `a8[( b8])`). Slurs with three or more notes are
    left for manual beaming.
    """
    pattern = re.compile(rf"({_NOTE_RE}8)\((\s*)({_NOTE_RE}8)\)")
    return pattern.sub(r"\1[(\2\3])", notes_content)


def modify_notes(notes_content, force_combine=False):
    """Apply visual adjustments to notes content for rendering.

    - If force_combine, merge line pairs by removing odd-numbered \\break markers
    - Inject accidentalStyle forget if no barlines
    - Hide clef after first line
    - Add hidden rests at start/end of lines for alignment
    - Beam slurred eighth-note groups
    """
    notes_content = _beam_slurred_eighths(notes_content)

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
                # Last line of this part: replace \break with an invisible
                # barline so inject_padding still sees a terminator (otherwise
                # this line gets no trailing padding and ends up shorter).
                if part_idx < n_parts - 1:
                    line = line.replace("\\break", "\\bar \"\"")
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
    """Count syllables' worth of notes: a slurred group counts as one."""
    lines = []
    for line in notes_content.splitlines():
        line = line.split("%")[0]
        lines.append(line)
    content = " ".join(lines)
    note_re = r"[a-g](is|es)?[',]*\d"
    total = len(re.findall(note_re, content))
    for m in re.finditer(r"\([^)]*\)", content):
        total -= len(re.findall(note_re, m.group()))
    return total


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
        if re.match(r"\\[a-zA-Z]", tok):
            continue
        count += 1
    return count


# Per-token widths in SVG units, used as initial estimates. Real widths
# vary per-document due to LilyPond's spring system — the algorithm probes
# with these defaults then calibrates a per-document scale factor.
# r4 (hidden rest) is the coarse pad; s16 (silent skip) is the fine-tune
# pad and saturates past ~7 per line.
_R4_WIDTH = 2.898
_S16_WIDTH_ALONE = 0.5123
_S16_WIDTH_AFTER_R4 = 0.725
_S16_MAX_PER_LINE = 7
# 1 mm of spread is below print resolution (~0.57 SVG units at default
# staff size). Used as the gate for the pass-5 iteration.
_TOLERANCE_1MM = 0.57


def _measure_staff_lengths(svg_path):
    """Return one staff length per system from a LilyPond-rendered SVG."""
    with open(svg_path) as f:
        svg = f.read()
    pat = re.compile(
        r'<line[^/]*stroke-width="0\.1000"[^/]*x1="([^"]+)"\s+y1="([^"]+)"'
        r'\s+x2="([^"]+)"\s+y2="([^"]+)"', re.S)
    horiz = []
    for m in pat.finditer(svg):
        x1, y1, x2, y2 = map(float, m.groups())
        if abs(y1 - y2) < 1e-6:
            horiz.append(x2 - x1)
    return [horiz[i * 5] for i in range(len(horiz) // 5)]


def _r4_padding(deficit, width=_R4_WIDTH):
    """Number of hidden r4 rests to add for the given deficit."""
    if deficit <= 0:
        return 0
    return max(0, round(deficit / width))


def _s16_padding(deficit, has_r4, scale=1.0):
    """Number of trailing s16 skips for fine-tune. `scale` adjusts the s16
    width by the same factor we calibrated for r4 — both track the same
    document-level spring stretch."""
    if deficit <= 0:
        return 0
    base = _S16_WIDTH_AFTER_R4 if has_r4 else _S16_WIDTH_ALONE
    n = round(deficit / (base * scale))
    return min(max(n, 0), _S16_MAX_PER_LINE)


def _inject_padding(modified_notes, paddings):
    """Insert padding before each \\break / \\bar terminator.
    Rest tokens (r4, etc.) are wrapped with `\\once \\hide Rest`; skip tokens
    (s16, etc.) pass through unchanged. If the line normally ends in a visible
    rest, padding is inserted before that rest so the rest stays at the end of
    the music; otherwise padding is appended just before the terminator."""
    parts = re.split(r'(\\break|\\bar\s+"[^"]*")', modified_notes)
    line_idx = 0
    for i in range(1, len(parts), 2):
        if line_idx >= len(paddings) or not paddings[line_idx]:
            line_idx += 1
            continue
        wrapped = [f"\\once \\hide Rest {t}" if t.startswith("r") else t
                   for t in paddings[line_idx]]
        pad = " ".join(wrapped)
        prev = parts[i-1]
        # Visible trailing rest: r<dur> at end, not preceded by `Rest ` (from
        # an immediately preceding `\\once \\hide Rest`).
        m = re.search(r'(?<!Rest )(\br\d+\.?\s*)$', prev)
        if m:
            pos = m.start(1)
            parts[i-1] = prev[:pos] + pad + " " + prev[pos:]
        else:
            parts[i-1] = prev.rstrip() + " " + pad + " "
        line_idx += 1
    return "".join(parts)


def _build_score_ly(modified_notes, lyrics_content, lyrics_mag=1.0):
    """Compose the full LilyPond document for a render pass."""
    lyrics_score = ""
    if lyrics_content.strip():
        lyrics_score = '    \\new Lyrics \\lyricsto "melody" { \\verse }'
    lyrics_font = ""
    if abs(lyrics_mag - 1.0) > 1e-6:
        y_lift = 0.5 * (lyrics_mag - 1.0)
        lyrics_font = (
            f"      \\override LyricText.font-size = "
            f"#(magnification->font-size {lyrics_mag})\n"
            f"      \\override LyricHyphen.thickness = #{1.3 * lyrics_mag}\n"
            f"      \\override LyricHyphen.length = #{0.66 * lyrics_mag}\n"
            f"      \\override LyricHyphen.extra-offset = #'(0 . {y_lift})\n"
        )
    return f"""\\version "2.24.0"

\\paper {{
  #(define fonts (set-global-fonts #:roman "FreeSerif"))
  paper-width = 1000\\mm
  line-width = 1000\\mm
  ragged-right = ##t
}}

\\header {{
  tagline = ##f
}}

{modified_notes}

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
      \\override LyricText.self-alignment-X = #CENTER
      \\override LyricHyphen.minimum-distance = #1.0
{lyrics_font}    }}
  }}
}}
"""


def render_svg_from_content(notes_content, lyrics_content, output_svg, composer=None, force_combine=False, uniform_staves=True, lyrics_mag=1.0):
    """Render notes and lyrics content to SVG.

    With uniform_staves=True, runs up to five LilyPond passes to align all
    staves to the longest natural staff:
      1. Render unpadded; measure natural lengths.
      2. Probe: pad with rough r4 counts using the default _R4_WIDTH; measure
         to calibrate the actual per-r4 width for THIS document.
      3. If the calibrated counts differ from the probe, re-render with them.
      4. s16 fine-tune, scaling s16 widths by the same calibrated factor.
      5. If spread is still over ~1 mm, iterate s16 using per-line observed
         widths from pass 4.

    Each pass is skipped when its preconditions don't apply (spread already
    within tolerance, no padding needed, etc.).
    """
    modified = modify_notes(notes_content, force_combine=force_combine)
    lyrics_content = lyrics_content.replace('\\"', '\u201c')

    out_dir = os.path.dirname(os.path.abspath(output_svg))
    os.makedirs(out_dir, exist_ok=True)
    svg_base = os.path.splitext(os.path.basename(output_svg))[0]
    combined_ly = os.path.join(out_dir, f"_combined_{svg_base}.ly")
    abs_svg = os.path.abspath(output_svg)
    cropped_svg = os.path.splitext(abs_svg)[0] + ".cropped.svg"

    def _do_render(content):
        with open(combined_ly, "w") as f:
            f.write(_build_score_ly(content, lyrics_content, lyrics_mag=lyrics_mag))
        result = subprocess.run(
            [_lilypond_bin(), "-dbackend=svg", "-dcrop", "-o",
             os.path.splitext(abs_svg)[0], combined_ly],
            capture_output=True, text=True, cwd=out_dir,
        )
        if result.returncode == 0 and os.path.exists(cropped_svg):
            os.replace(cropped_svg, abs_svg)
        return result.returncode == 0

    # Pass 1: render unpadded, measure natural lengths.
    if not _do_render(modified):
        return False
    if not uniform_staves:
        return True

    L0 = _measure_staff_lengths(abs_svg)
    if not L0 or max(L0) - min(L0) <= 0.01:
        return True

    # Pass 2: probe — pad with the default _R4_WIDTH guess so we can observe
    # the real per-r4 width of THIS document (it varies 2.9..5.7 across docs
    # due to LilyPond's spring system; a fixed constant is unreliable).
    target = max(L0)
    r4_counts = [_r4_padding(target - l) for l in L0]
    scale = 1.0
    if any(r4_counts):
        paddings = [["r4"] * n for n in r4_counts]
        padded = _inject_padding(modified, paddings)
        if not _do_render(padded):
            return False
        L1 = _measure_staff_lengths(abs_svg)
        if not L1:
            return True

        # Calibrate per-r4 width from observed lengthening.
        samples = [(L1[i] - L0[i]) / r4_counts[i]
                   for i in range(len(L0)) if r4_counts[i] > 0]
        w_r4 = sum(samples) / len(samples) if samples else _R4_WIDTH
        scale = w_r4 / _R4_WIDTH

        # Pass 3: re-pad using calibrated width. Target the natural max so
        # per-line overshoot in pass 2 doesn't inflate the target.
        new_r4 = [_r4_padding(target - L0[i], w_r4) for i in range(len(L0))]
        if new_r4 != r4_counts:
            paddings = [["r4"] * n for n in new_r4]
            padded = _inject_padding(modified, paddings)
            if not _do_render(padded):
                return False
            L1 = _measure_staff_lengths(abs_svg)
            if not L1:
                return True
            r4_counts = new_r4
    else:
        # All deficits are sub-r4 — skip r4 padding, let s16 finish.
        L1 = L0

    if max(L1) - min(L1) <= 0.01:
        return True

    # Pass 4: s16 fine-tune, using the r4-calibrated scale for s16 widths.
    target = max(L1)
    s16_counts = [_s16_padding(target - l, r4_counts[i] > 0, scale)
                  for i, l in enumerate(L1)]
    if not any(s16_counts):
        return True
    paddings = [["r4"] * r4 + ["s16"] * s16
                for r4, s16 in zip(r4_counts, s16_counts)]
    padded = _inject_padding(modified, paddings)
    if not _do_render(padded):
        return False

    # Pass 5: if spread is still over 1 mm, iterate s16 using per-line
    # observed widths. s16 width is nonlinear per-line, so the r4-calibrated
    # scale doesn't always nail it on the first try.
    L2 = _measure_staff_lengths(abs_svg)
    if not L2 or max(L2) - min(L2) <= _TOLERANCE_1MM:
        return True

    target = max(L2)
    additional = []
    for i, l in enumerate(L2):
        deficit = target - l
        if deficit <= 0:
            additional.append(0)
            continue
        # Prefer observed s16 width if we have a sample for this line.
        if s16_counts[i] > 0 and L2[i] > L1[i]:
            w_s16 = (L2[i] - L1[i]) / s16_counts[i]
        else:
            base = _S16_WIDTH_AFTER_R4 if r4_counts[i] > 0 else _S16_WIDTH_ALONE
            w_s16 = base * scale
        n = round(deficit / w_s16) if w_s16 > 0 else 0
        # Stay within the s16 per-line cap (it really does saturate past ~7).
        n = min(max(n, 0), _S16_MAX_PER_LINE - s16_counts[i])
        additional.append(n)
    if not any(additional):
        return True
    new_s16 = [s16_counts[i] + additional[i] for i in range(len(s16_counts))]
    paddings = [["r4"] * r4 + ["s16"] * s16
                for r4, s16 in zip(r4_counts, new_s16)]
    padded = _inject_padding(modified, paddings)
    return _do_render(padded)


def render_svg(notes_path, lyrics_path, output_svg, composer=None, split_style="default", lyrics_mag=1.0):
    """Render notes and lyrics files to SVG.

    Args:
        notes_path: Path to .ly file containing melody definition
        lyrics_path: Path to .ly file containing verse definition (or None)
        output_svg: Path for output SVG file
        composer: Optional composer attribution string
        split_style: One of "default", "single slide", "multi slide", "combine lines"
        lyrics_mag: Magnification factor for lyric text (1.0 = unchanged)
    """
    with open(notes_path) as f:
        notes_content = f.read()

    lyrics_content = ""
    if lyrics_path and os.path.exists(lyrics_path):
        with open(lyrics_path) as f:
            lyrics_content = f.read()

    if lyrics_content.strip():
        note_count = _count_pitched_notes(notes_content)
        syllable_count = _count_syllables(lyrics_content)
        if note_count != syllable_count:
            import warnings
            warnings.warn(
                f"{notes_path}: note/syllable mismatch: "
                f"{note_count} notes vs {syllable_count} syllables"
            )

    return render_svg_from_content(
        notes_content, lyrics_content, output_svg,
        composer=composer,
        force_combine=split_style == "combine lines",
        lyrics_mag=lyrics_mag,
    )


def _render_one(args):
    """Render a single lyrics file. Returns (label, success)."""
    lyrics_path, no_combine, lyrics_mag = args
    psalm_dir = os.path.dirname(lyrics_path)
    psalm_name = os.path.basename(psalm_dir)

    basename = os.path.basename(lyrics_path)
    verse_num = basename.replace("lyrics_", "").replace(".ly", "")

    # Use verse-specific notes file if it exists, otherwise fall back to notes.ly
    verse_notes_path = os.path.join(psalm_dir, f"notes_{verse_num}.ly")
    notes_path = verse_notes_path if os.path.exists(verse_notes_path) else os.path.join(psalm_dir, "notes.ly")

    if not os.path.exists(notes_path):
        return (f"{psalm_name}", "SKIP")

    composer = None
    split_style = "default"
    yaml_path = os.path.join(psalm_dir, "song.yaml")
    if os.path.exists(yaml_path):
        import yaml
        with open(yaml_path) as f:
            meta = yaml.safe_load(f) or {}
        composer = meta.get("composer")
        split_style = meta.get("split_style", "default")

    if no_combine:
        split_style = "single slide"

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
            ok = render_svg_from_content(notes_part, lyrics_part, svg_path, composer=comp, lyrics_mag=lyrics_mag)
            if not ok:
                all_ok = False
        label = f"{psalm_name} v{verse_num} ({len(note_parts)} parts)"
        return (label, "OK" if all_ok else "FAILED")
    else:
        svg_path = os.path.join(psalm_dir, f"{verse_num}.svg")
        ok = render_svg(notes_path, lyrics_path, svg_path, composer=composer, split_style=split_style, lyrics_mag=lyrics_mag)
        return (f"{psalm_name} v{verse_num}", "OK" if ok else "FAILED")


def main():
    """Render all psalms in the lilypond directory."""
    parser = argparse.ArgumentParser(description="Render all psalm SVGs")
    parser.add_argument("-j", "--jobs", type=int, default=os.cpu_count(), help="Parallel workers (default: nproc)")
    parser.add_argument("--psalm", help="Process only this psalm (e.g. psalm102)")
    parser.add_argument("--check", action="store_true", help="Check note/syllable counts without rendering")
    parser.add_argument("--no-combine", action="store_true", help="Force single-slide rendering (no line-pair combining)")
    parser.add_argument("--lyrics-mag", type=float, default=1.0, help="Lyrics font magnification (1.0 = unchanged)")
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
            basename = os.path.basename(lyrics_path).replace("lyrics_", "").replace(".ly", "")
            verse_notes_path = os.path.join(song_dir, f"notes_{basename}.ly")
            notes_path = verse_notes_path if os.path.exists(verse_notes_path) else os.path.join(song_dir, "notes.ly")
            if not os.path.exists(notes_path):
                continue
            with open(notes_path) as f:
                note_count = _count_pitched_notes(f.read())
            with open(lyrics_path) as f:
                syllable_count = _count_syllables(f.read())
            label = f"{song_name} v{basename}"
            if note_count == syllable_count:
                ok += 1
            else:
                failed += 1
                print(f"  MISMATCH {label}: {note_count} notes vs {syllable_count} syllables")
        print(f"Checked {ok + failed}, {failed} mismatches")
        return

    print(f"Found {len(lyrics_files)} lyrics files")

    work = [(lf, args.no_combine, args.lyrics_mag) for lf in lyrics_files]
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

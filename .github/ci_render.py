#!/usr/bin/env python3
"""CI helper for rendering song previews and generating PR comments.

Subcommands:
  render   Render SVGs and convert to PNGs for changed song dirs.
  comment  Generate markdown PR comment body from rendered PNGs.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile


def natural_sort_key(path):
    """Sort key that handles numeric parts naturally (1_p0 before 1_p10)."""
    return [
        int(part) if part.isdigit() else part.lower()
        for part in re.split(r'(\d+)', path)
    ]


def git_show_file(commit, repo_path):
    """Get file contents at a specific commit. Returns None if file doesn't exist."""
    result = subprocess.run(
        ["git", "show", f"{commit}:{repo_path}"],
        capture_output=True,
    )
    if result.returncode == 0:
        return result.stdout
    return None


def render_base_pngs(base_sha, song_dir):
    """Render PNGs for a song directory at a base commit. Returns dict of {basename: png_path}."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_dir = os.path.dirname(script_dir)
    sys.path.insert(0, os.path.join(repo_dir, "png2ly"))
    import render_ly

    song = os.path.basename(song_dir)
    repo_song_dir = f"lilypond/{song}"

    base_dir = tempfile.mkdtemp(prefix=f"base_{song}_")

    # Retrieve notes.ly at base commit
    notes_data = git_show_file(base_sha, f"{repo_song_dir}/notes.ly")
    if notes_data is None:
        shutil.rmtree(base_dir)
        return {}

    notes_path = os.path.join(base_dir, "notes.ly")
    with open(notes_path, "wb") as f:
        f.write(notes_data)

    # Retrieve song.yaml for composer/split_style
    composer = None
    split_style = "default"
    yaml_data = git_show_file(base_sha, f"{repo_song_dir}/song.yaml")
    if yaml_data is not None:
        import yaml
        meta = yaml.safe_load(yaml_data) or {}
        composer = meta.get("composer")
        split_style = meta.get("split_style", "default")

    # Retrieve all lyrics and verse-specific notes files at base commit
    result = subprocess.run(
        ["git", "ls-tree", "--name-only", base_sha, f"{repo_song_dir}/"],
        capture_output=True, text=True,
    )
    lyrics_files = []
    if result.returncode == 0:
        for line in result.stdout.splitlines():
            filename = os.path.basename(line)
            if (filename.startswith("lyrics_") or filename.startswith("notes_")) and filename.endswith(".ly"):
                data = git_show_file(base_sha, f"{repo_song_dir}/{filename}")
                if data is not None:
                    lpath = os.path.join(base_dir, filename)
                    with open(lpath, "wb") as f:
                        f.write(data)
                    if filename.startswith("lyrics_"):
                        lyrics_files.append(lpath)

    base_pngs = {}
    for lyrics_path in sorted(lyrics_files):
        verse_num = os.path.basename(lyrics_path).replace("lyrics_", "").replace(".ly", "")

        # Use verse-specific notes if available, otherwise fall back to notes.ly
        verse_notes_path = os.path.join(base_dir, f"notes_{verse_num}.ly")
        cur_notes_path = verse_notes_path if os.path.exists(verse_notes_path) else notes_path

        with open(cur_notes_path) as f:
            raw_notes = f.read()

        break_count = raw_notes.count("\\break")
        n_parts = render_ly._effective_n_parts(split_style, break_count)

        if n_parts > 1:
            with open(lyrics_path) as f:
                raw_lyrics = f.read()
            raw_lyrics = raw_lyrics.replace('\\"', '\u201c')
            total_note_lines = render_ly._count_note_lines(raw_notes)
            note_parts = render_ly._split_notes(raw_notes, n_parts)
            lyric_parts = render_ly._split_lyrics(raw_lyrics, n_parts, total_note_lines)

            for part_idx, (notes_part, lyrics_part) in enumerate(zip(note_parts, lyric_parts)):
                svg_path = os.path.join(base_dir, f"{verse_num}_p{part_idx}.svg")
                comp = composer if part_idx == 0 else None
                render_ly.render_svg_from_content(notes_part, lyrics_part, svg_path, composer=comp)
        else:
            svg_path = os.path.join(base_dir, f"{verse_num}.svg")
            render_ly.render_svg(cur_notes_path, lyrics_path, svg_path, composer=composer, split_style=split_style)

    # Convert SVGs to PNGs
    for f in os.listdir(base_dir):
        if f.endswith(".svg"):
            svg_path = os.path.join(base_dir, f)
            png_path = svg_path.rsplit(".", 1)[0] + ".png"
            result = subprocess.run(
                ["rsvg-convert", "-d", "300", "-p", "300", svg_path, "-o", png_path],
                capture_output=True, text=True,
            )
            if result.returncode == 0:
                basename = os.path.basename(png_path).replace(".png", "")
                base_pngs[basename] = png_path

    return base_pngs


def make_diff_image(old_png, new_png, out_png):
    """Create a diff image: red for removed pixels, green for added, gray for unchanged."""
    from PIL import Image
    import numpy as np

    new_img = Image.open(new_png).convert("RGBA")
    new_arr = np.array(new_img)

    if old_png and os.path.exists(old_png):
        old_img = Image.open(old_png).convert("RGBA")
        h = max(old_img.height, new_img.height)
        w = max(old_img.width, new_img.width)
        old_arr = np.zeros((h, w, 4), dtype=np.uint8)
        old_arr[:old_img.height, :old_img.width] = np.array(old_img)
        padded_new = np.zeros((h, w, 4), dtype=np.uint8)
        padded_new[:new_img.height, :new_img.width] = new_arr
        new_arr = padded_new
    else:
        h, w = new_arr.shape[:2]
        old_arr = np.zeros((h, w, 4), dtype=np.uint8)

    old_opaque = old_arr[:, :, 3] > 128
    new_opaque = new_arr[:, :, 3] > 128

    out = np.full((h, w, 4), 255, dtype=np.uint8)

    both = old_opaque & new_opaque
    out[both] = [180, 180, 180, 255]

    removed = old_opaque & ~new_opaque
    out[removed] = [220, 40, 40, 255]

    added = ~old_opaque & new_opaque
    out[added] = [40, 180, 40, 255]

    Image.fromarray(out).save(out_png)


def cmd_render(args):
    """Render SVGs and convert to PNGs for the given song directories."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_dir = os.path.dirname(script_dir)
    render_ly = os.path.join(repo_dir, "png2ly", "render_ly.py")

    all_pngs = []
    for song_dir in args.dirs:
        song_dir = song_dir.strip()
        if not song_dir:
            continue
        song = os.path.basename(song_dir)
        print(f"Rendering {song}...", file=sys.stderr)

        # Render base version if diffing
        base_pngs = {}
        if args.base_sha:
            print(f"  Rendering base version at {args.base_sha[:8]}...", file=sys.stderr)
            base_pngs = render_base_pngs(args.base_sha, song_dir)

        # Render current version
        subprocess.run(
            [sys.executable, render_ly, "--psalm", song],
            check=False,
            stdout=sys.stderr,
        )

        for svg in sorted(
            (os.path.join(song_dir, f) for f in os.listdir(song_dir) if f.endswith(".svg")),
            key=natural_sort_key,
        ):
            basename = os.path.basename(svg).rsplit(".", 1)[0]
            png = os.path.join(song_dir, basename + ".png")

            result = subprocess.run(
                ["rsvg-convert", "-d", "300", "-p", "300", svg, "-o", png],
                capture_output=True, text=True,
            )
            if result.returncode != 0:
                print(f"  rsvg-convert failed for {svg}: {result.stderr}", file=sys.stderr)
                continue

            all_pngs.append(png)

            if args.base_sha:
                old_png = base_pngs.get(basename)
                diff_png = os.path.join(song_dir, basename + "_diff.png")
                make_diff_image(old_png, png, diff_png)
                all_pngs.append(diff_png)

        # Clean up base temp files
        for p in base_pngs.values():
            tmp_dir = os.path.dirname(p)
            if os.path.isdir(tmp_dir):
                shutil.rmtree(tmp_dir)

    all_pngs.sort(key=natural_sort_key)
    for png in all_pngs:
        print(png)


def cmd_comment(args):
    """Generate markdown PR comment body from a list of PNG paths."""
    pngs = [line.strip() for line in sys.stdin if line.strip().endswith(".png")]
    if not pngs:
        return

    pngs.sort(key=natural_sort_key)

    body = "## Rendered Previews\n\n"
    last_song = ""
    for png in pngs:
        parts = png.split("/")
        song = parts[-2]
        filename = os.path.basename(png).replace(".png", "")
        is_diff = filename.endswith("_diff")
        verse = filename.replace("_diff", "")
        if song != last_song:
            body += f"### {song}\n"
            last_song = song
        png_url = f"https://github.com/{args.repo}/raw/{args.branch}/{png}"
        if is_diff:
            body += f"**Verse {verse} (diff)**\n![{song} v{verse} diff]({png_url})\n"
        else:
            body += f"**Verse {verse}**\n![{song} v{verse}]({png_url})\n"

    body += "\n\U0001f7e2 Added \U0001f534 Removed \u26aa Unchanged\n"
    print(body)


def main():
    parser = argparse.ArgumentParser(description="CI helper for song rendering")
    sub = parser.add_subparsers(dest="command", required=True)

    p_render = sub.add_parser("render", help="Render SVGs and convert to PNGs")
    p_render.add_argument("dirs", nargs="+", help="Song directories to render")
    p_render.add_argument("--base-sha", help="Base commit SHA to diff against")

    p_comment = sub.add_parser("comment", help="Generate PR comment markdown (reads PNGs from stdin)")
    p_comment.add_argument("--repo", required=True, help="GitHub owner/repo")
    p_comment.add_argument("--branch", required=True, help="Git branch for raw URLs")

    args = parser.parse_args()
    if args.command == "render":
        cmd_render(args)
    elif args.command == "comment":
        cmd_comment(args)


if __name__ == "__main__":
    main()

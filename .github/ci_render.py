#!/usr/bin/env python3
"""CI helper for rendering song previews and generating PR comments.

Subcommands:
  render   Render SVGs and convert to PNGs for changed song dirs.
  comment  Generate markdown PR comment body from rendered PNGs.
"""

import argparse
import json
import os
import re
import subprocess
import sys


def natural_sort_key(path):
    """Sort key that handles numeric parts naturally (1_p0 before 1_p10)."""
    return [
        int(part) if part.isdigit() else part.lower()
        for part in re.split(r'(\d+)', path)
    ]


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

        subprocess.run(
            [sys.executable, render_ly, "--psalm", song],
            check=False,
            stdout=sys.stderr,
        )

        for svg in sorted(
            (os.path.join(song_dir, f) for f in os.listdir(song_dir) if f.endswith(".svg")),
            key=natural_sort_key,
        ):
            png = svg.rsplit(".", 1)[0] + ".png"
            result = subprocess.run(
                ["rsvg-convert", svg, "-o", png],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                all_pngs.append(png)
            else:
                print(f"  rsvg-convert failed for {svg}: {result.stderr}", file=sys.stderr)

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
        verse = os.path.basename(png).replace(".png", "")
        if song != last_song:
            body += f"### {song}\n"
            last_song = song
        png_url = f"https://github.com/{args.repo}/raw/{args.branch}/{png}"
        body += f"**Verse {verse}**\n![{song} v{verse}]({png_url})\n"

    print(body)


def main():
    parser = argparse.ArgumentParser(description="CI helper for song rendering")
    sub = parser.add_subparsers(dest="command", required=True)

    p_render = sub.add_parser("render", help="Render SVGs and convert to PNGs")
    p_render.add_argument("dirs", nargs="+", help="Song directories to render")

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

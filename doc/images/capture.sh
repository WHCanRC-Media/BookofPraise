#!/usr/bin/env bash
#
# capture.sh — grab a screenshot of the Book of Praise app for the user guide.
#
# Usage:
#   doc/images/capture.sh [psalm|hymn] [NUMBER] [--editor] [-o OUTPUT.png]
#
# Examples:
#   doc/images/capture.sh                              # psalm 1 -> doc/images/psalm-1.png
#   doc/images/capture.sh hymn 7                       # hymn 7  -> doc/images/hymn-7.png
#   doc/images/capture.sh psalm 1 --editor -o e.png    # editor panel open
#
# Requires a KDE Plasma (Wayland) session: uses KWin scripting to focus the
# window and Spectacle to capture it.
set -euo pipefail

# ── Locate the repo root (this script lives in doc/images/) ─────────
REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$REPO_ROOT"

# ── Parse arguments ─────────────────────────────────────────────────
KIND=psalm
NUM=1
OUT=""
EDITOR=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        psalm|hymn) KIND=$1; shift ;;
        --editor)   EDITOR=1; shift ;;
        -o)         OUT=$2;  shift 2 ;;
        [0-9]*)     NUM=$1;  shift ;;
        *) echo "capture.sh: unknown argument '$1'" >&2; exit 2 ;;
    esac
done
if [[ -z "$OUT" ]]; then
    OUT="doc/images/${KIND}-${NUM}.png"
    [[ $EDITOR == 1 ]] && OUT="doc/images/${KIND}-${NUM}-editor.png"
fi
case "$OUT" in /*) ;; *) OUT="$REPO_ROOT/$OUT" ;; esac

BIN="$REPO_ROOT/target/release/bop"
LOG=$(mktemp -t bop-capture-XXXX.log)

# ── Assemble the app's command-line arguments ───────────────────────
BOP_ARGS=("--${KIND}" "$NUM")
[[ $EDITOR == 1 ]] && BOP_ARGS+=(--editor)

# ── Build (a no-op when already up to date) ─────────────────────────
echo "Building release binary..."
cargo build --release

# ── Launch the app with the requested song loaded ───────────────────
echo "Launching bop ${BOP_ARGS[*]} ..."
"$BIN" "${BOP_ARGS[@]}" >"$LOG" 2>&1 &
BOP_PID=$!
cleanup() { kill "$BOP_PID" 2>/dev/null || true; rm -f "$LOG"; }
trap cleanup EXIT

# ── Wait for the window to map and paint its first slide ────────────
for _ in $(seq 1 80); do
    kill -0 "$BOP_PID" 2>/dev/null || { echo "bop exited early; log:" >&2; cat "$LOG" >&2; exit 1; }
    grep -q "Rendering" "$LOG" 2>/dev/null && break
    sleep 0.25
done
sleep 2.5   # give the first slide (and the editor panel, if any) time to settle

# ── Focus & raise the bop window via a KWin script ──────────────────
KWIN_JS=$(mktemp -t bop-focus-XXXX.js)
cat >"$KWIN_JS" <<'EOF'
workspace.windowList().forEach(function (w) {
    var cls = String(w.resourceClass || "");
    if (cls.indexOf("bop") !== -1 || w.caption === "Book of Praise") {
        w.minimized = false;
        workspace.activeWindow = w;
    }
});
EOF
PLUGIN="bop-focus-$$"
qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript "$KWIN_JS" "$PLUGIN" >/dev/null
qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.start
qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript "$PLUGIN" >/dev/null 2>&1 || true
rm -f "$KWIN_JS"
sleep 0.5

# ── Capture the focused window ──────────────────────────────────────
echo "Capturing..."
spectacle -b -n -a -S -o "$OUT"

echo "Saved screenshot: $OUT"

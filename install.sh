#!/usr/bin/env bash
# HexLab — Omarchy / Linux install & run.
# Usage:  curl -fsSL https://raw.githubusercontent.com/SoMaCoSF/hexlab-rust/main/install.sh | bash
#    or:  ./install.sh [dir]
set -euo pipefail
DIR="${1:-$HOME/hexlab-rust}"

command -v git >/dev/null || { echo "MISSING: git (sudo pacman -S git)"; exit 1; }
if ! command -v cargo >/dev/null; then
  echo "Rust not found — installing via rustup…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
fi

if [ -d "$DIR/.git" ]; then
  echo "Updating $DIR"; git -C "$DIR" pull --ff-only
else
  echo "Cloning SoMaCoSF/hexlab-rust → $DIR"
  git clone https://github.com/SoMaCoSF/hexlab-rust.git "$DIR"
fi
cd "$DIR"

echo "Building hexlab + hexlab-tui (release)…"
cargo build --release -p hexlab-cli -p hexlab-tui
BIN="$DIR/target/release"

echo "— Smoke test —"
"$BIN/hexlab" bom woodshop | head -4
"$BIN/hexlab" city preset recon -o /tmp/hexlab-city.json
"$BIN/hexlab" city telemetry /tmp/hexlab-city.json
"$BIN/hexlab" spoken "create a bom for a woodshop and stock it" --city /tmp/hexlab-city.json

echo
echo "HexLab is built. Run it:"
echo "  $BIN/hexlab bom woodshop"
echo "  $BIN/hexlab city preset carrier -o city.json"
echo "  $BIN/hexlab-tui --city city.json      # full-screen TUI"
echo "Omarchy / Hyprland: pin it to a workspace —"
echo "  hyprctl dispatch exec '[workspace 1] kitty -e $BIN/hexlab-tui --city city.json'"
echo "Tauri desktop shell (Arch: sudo pacman -S webkit2gtk-4.1):"
echo "  cargo install tauri-cli --version '^2' && cd apps/hexlab-tauri && cargo tauri build"

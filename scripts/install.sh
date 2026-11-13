#!/bin/sh
# Installs GitVault (the `gitvault` command and its short alias `gv`) on a Linux server or desktop.
#
#   curl -fsSL https://raw.githubusercontent.com/adhil72/gitvault/main/scripts/install.sh | sh
#
# Debian/Ubuntu: installs the signed apt package (also pulls in git and gh) and updates with apt.
# Other Linux: builds the CLI with cargo.
set -eu

BASE="https://github.com/adhil72/gitvault-website/releases/latest/download"
KEYRING="/usr/share/keyrings/gitvault-archive-keyring.gpg"

say() { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

# Runs a command as root, through sudo when we are not root.
as_root() {
  if [ "$(id -u)" -eq 0 ]; then "$@"; elif have sudo; then sudo "$@"; else fail "run this as root or install sudo"; fi
}

install_apt() {
  have curl || as_root apt-get install -y curl
  as_root curl -fsSL "$BASE/gitvault-archive-keyring.gpg" -o "$KEYRING"
  echo "deb [signed-by=$KEYRING] $BASE ./" | as_root tee /etc/apt/sources.list.d/gitvault.list >/dev/null
  as_root apt-get update
  as_root apt-get install -y git-vault
}

install_cargo() {
  have git || fail "git is required"
  have cargo || fail "no apt and no cargo found: install Rust (https://rustup.rs) and run this again"
  cargo install --locked --git https://github.com/adhil72/gitvault gitvault-cli
  bin="${CARGO_HOME:-$HOME/.cargo}/bin"
  ln -sf "$bin/gitvault" "$bin/gv"
}

if have apt-get; then install_apt; else install_cargo; fi

say ""
say "GitVault is installed. Next:"
have gh || say "  1. install the GitHub CLI: https://cli.github.com"
say "  gh auth login          # once; gv reuses it"
say "  gv vault create --name Personal"
say "  cd your-project && gv trust"

#!/usr/bin/env sh
# GitVault v2 installer: requires `git` and `gh` CLIs to be installed first.
#
# Remote install:
#   curl -fsSL https://raw.githubusercontent.com/adhil72/gitvault/main/scripts/install.sh | sh
# Local checkout install:
#   ./scripts/install.sh
set -eu

REPO_URL="https://github.com/adhil72/gitvault"
RELEASE_BASE="https://github.com/adhil72/gitvault-website/releases/latest/download"

fail() {
  printf '✗ %s\n' "$1" >&2
  exit 1
}

have() {
  command -v "$1" >/dev/null 2>&1
}

have git || fail "git is required. Install Git from https://git-scm.com/downloads and retry."
have gh || fail "GitHub CLI (gh) is required. Install gh from https://cli.github.com and retry."

detect_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)
  case "$os:$arch" in
    linux:x86_64)   printf '%s' "gitvault-x86_64-unknown-linux-gnu.tar.gz" ;;
    linux:aarch64)  printf '%s' "gitvault-aarch64-unknown-linux-gnu.tar.gz" ;;
    darwin:arm64)   printf '%s' "gitvault-aarch64-apple-darwin.tar.gz" ;;
    darwin:x86_64)  printf '%s' "gitvault-x86_64-apple-darwin.tar.gz" ;;
    *)              printf '' ;;
  esac
}

install_from_release() {
  asset=$(detect_target)
  [ -n "$asset" ] || return 1
  have curl || return 1
  have tar || return 1

  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT INT TERM
  if curl -fsSL "$RELEASE_BASE/$asset" -o "$tmp/$asset"; then
    bin_dir="${HOME}/.local/bin"
    mkdir -p "$bin_dir"
    tar -xzf "$tmp/$asset" -C "$tmp"
    install -m 0755 "$tmp/gitvault" "$bin_dir/gitvault"
    ln -sf "$bin_dir/gitvault" "$bin_dir/gv"
    printf '✓ Installed gitvault and alias '\''gv'\'' to %s\n' "$bin_dir"
    case ":${PATH}:" in
      *":$bin_dir:"*) ;;
      *) printf '! Add %s to your PATH if not already present.\n' "$bin_dir" ;;
    esac
    return 0
  fi
  return 1
}

install_with_cargo() {
  have cargo || fail "cargo not found (and prebuilt release unavailable). Install Rust from https://rustup.rs and retry."
  if [ -n "${LOCAL_ROOT:-}" ]; then
    cargo install --path "$LOCAL_ROOT" --locked --force
  else
    cargo install --git "$REPO_URL" --locked --force
  fi
  bin_dir="${CARGO_HOME:-$HOME/.cargo}/bin"
  if [ -x "$bin_dir/gitvault" ]; then
    ln -sf "$bin_dir/gitvault" "$bin_dir/gv"
    printf '✓ Installed gitvault and alias '\''gv'\'' to %s\n' "$bin_dir"
  fi
}

LOCAL_ROOT=""
case "$0" in
  */*)
    script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
    root_dir=$(CDPATH='' cd -- "$script_dir/.." && pwd)
    if [ -f "$root_dir/Cargo.toml" ] && grep -q '^name = "gitvault"' "$root_dir/Cargo.toml"; then
      LOCAL_ROOT="$root_dir"
    fi
    ;;
esac

if [ -n "$LOCAL_ROOT" ]; then
  printf '◆ Building and installing gitvault v2 from local checkout...\n'
  install_with_cargo
elif ! install_from_release; then
  printf '◆ Installing gitvault v2 from source via cargo...\n'
  install_with_cargo
fi

if ! gh auth status >/dev/null 2>&1; then
  printf '! GitHub CLI is not signed in yet. Run: gh auth login\n'
fi

printf '\nOptional (recommended): add this line to ~/.bashrc or ~/.zshrc so '\''gitvault load'\'' updates your active shell in RAM:\n'
printf '  eval "$(gitvault shell-init)"\n'

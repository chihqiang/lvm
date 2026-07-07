#!/bin/sh
set -eu

# ============================================================
# Configuration
# ============================================================

BIN_NAME="lvm"
INSTALL_DIR="${LVM_INSTALL_DIR:-/usr/local/bin}"
LVM_DOWNLOAD_URL="${LVM_DOWNLOAD_URL:-https://github.com/chihqiang/lvm/releases/latest/download}"

# ============================================================
# Utilities
# ============================================================

usage() {
  cat <<USAGE
lvm installer for macOS and Linux.

Usage:
  curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | sh

Environment:
  LVM_INSTALL_DIR       Install directory. Default: /usr/local/bin
  LVM_DOWNLOAD_URL      Custom release asset base URL. Default: latest release download URL

Examples:
  curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | sudo sh
  curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | LVM_INSTALL_DIR=/opt/bin sh
USAGE
}

say() {
  printf '%s\n' "$*"
}

fail() {
  printf 'lvm install: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

download() {
  url="$1"
  out="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$out"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$out"
  else
    fail "curl or wget is required"
  fi
}

detect_platform() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin) platform="apple-darwin" ;;
    Linux)  platform="unknown-linux-musl" ;;
    *)      fail "unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64)  cpu="x86_64" ;;
    aarch64|arm64) cpu="aarch64" ;;
    *)             fail "unsupported CPU architecture: $arch" ;;
  esac

  printf '%s-%s' "$cpu" "$platform"
}

# Resolve sudo: use sudo only when the target directory is not writable.
resolve_sudo() {
  if [ -d "$INSTALL_DIR" ]; then
    if [ ! -w "$INSTALL_DIR" ] ||
      { [ -e "$INSTALL_DIR/$BIN_NAME" ] && [ ! -w "$INSTALL_DIR/$BIN_NAME" ]; }; then
      need_cmd sudo
      sudo_cmd="sudo"
    fi
  elif ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    need_cmd sudo
    sudo mkdir -p "$INSTALL_DIR"
    sudo_cmd="sudo"
  fi
}

# ============================================================
# Main
# ============================================================

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

target="$(detect_platform)"
archive_name="${BIN_NAME}-${target}.tar.gz"
archive_url="${LVM_DOWNLOAD_URL}/${archive_name}"

tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t lvm-install)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

say "Installing ${BIN_NAME} for ${target}"
say "Release URL: ${LVM_DOWNLOAD_URL}"
say "Install dir: ${INSTALL_DIR}"
say ""

download "$archive_url" "$tmpdir/$archive_name"
say "Downloaded ${archive_name}"

say "Extracting..."
tar xzf "$tmpdir/$archive_name" -C "$tmpdir"

sudo_cmd=""
resolve_sudo

# Atomic install: copy to a temp stage, then mv into place
stage="$INSTALL_DIR/.${BIN_NAME}.$$"
trap 'rm -rf "$tmpdir"; rm -f "$stage" 2>/dev/null || true' EXIT INT TERM

$sudo_cmd cp "$tmpdir/$BIN_NAME" "$stage"
$sudo_cmd chmod 755 "$stage"
$sudo_cmd mv "$stage" "$INSTALL_DIR/$BIN_NAME"

say ""
say "Installed:"
"$INSTALL_DIR/$BIN_NAME" --version || true

# PATH hint
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    say ""
    say "Add $INSTALL_DIR to PATH to run ${BIN_NAME} from any terminal."
    ;;
esac

# Shell integration hint
say ""
say "Add the following to your shell config (~/.bashrc or ~/.zshrc):"
say ""
say '  eval "$(lvm env)"'
say '  eval "$(lvm hook)"'

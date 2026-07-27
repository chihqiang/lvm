#!/bin/sh
set -eu

# ============================================================
# Configuration
# ============================================================

BIN_NAME="lvm"
INSTALL_DIR="${LVM_INSTALL_DIR:-/usr/local/bin}"

# Version to install. Default: latest release.
# Pin to a specific version: export LVM_VERSION=v0.0.7
VERSION="${LVM_VERSION:-latest}"

if [ "$VERSION" = "latest" ]; then
  RELEASE_BASE="https://github.com/chihqiang/lvm/releases/latest/download"
else
  # Strip leading 'v' if present for consistency
  VER_STR="${VERSION#v}"
  RELEASE_BASE="https://github.com/chihqiang/lvm/releases/download/v${VER_STR}"
fi

# Allow full override of the download base URL
DOWNLOAD_BASE="${LVM_DOWNLOAD_URL:-$RELEASE_BASE}"

# ============================================================
# Utilities
# ============================================================

usage() {
  cat <<USAGE
lvm installer for macOS and Linux.

Usage:
  curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | sh

Environment:
  LVM_VERSION           Version to install (e.g. v0.0.7). Default: latest release.
  LVM_INSTALL_DIR       Install directory. Default: /usr/local/bin
  LVM_DOWNLOAD_URL      Custom release asset base URL (overrides version auto-resolution).

Examples:
  curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | sudo sh
  LVM_VERSION=v0.0.7 curl -fsSL https://raw.githubusercontent.com/chihqiang/lvm/main/install.sh | sh
  LVM_INSTALL_DIR=~/.local/bin curl -fsSL ... | sh
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
    # Show progress bar when stdout is a terminal, otherwise silent
    if [ -t 1 ]; then
      curl -fSL# "$url" -o "$out"
    else
      curl -fsSL "$url" -o "$out"
    fi
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

# Verify SHA256 checksum of the downloaded archive.
# Falls back to checking only the basename match in SHASUMS256.txt.
verify_checksum() {
  archive_path="$1"
  archive_file="$(basename "$archive_path")"

  checksums_url="${DOWNLOAD_BASE}/checksums.txt"
  checksums_file="$(mktemp -t lvm-shasums.XXXXXX 2>/dev/null || mktemp /tmp/lvm-shasums.XXXXXX)"

  if ! download "$checksums_url" "$checksums_file" 2>/dev/null; then
    rm -f "$checksums_file"
    say "  Warning: checksum file not available, skipping verification"
    return 0
  fi

  # Find the matching entry: the archive filename is the last field
  expected=$(grep -F "$archive_file" "$checksums_file" | awk '{print $1}' | head -1)
  rm -f "$checksums_file"

  if [ -z "$expected" ]; then
    say "  Warning: no checksum found for ${archive_file}, skipping verification"
    return 0
  fi

  # Detect platform-specific sha256 tool
  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$archive_path" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$archive_path" | awk '{print $1}')
  elif command -v openssl >/dev/null 2>&1; then
    actual=$(openssl dgst -sha256 "$archive_path" | awk '{print $NF}')
  else
    say "  Warning: no sha256 tool found, skipping verification"
    return 0
  fi

  if [ "$expected" != "$actual" ]; then
    fail "checksum mismatch for ${archive_file} (expected ${expected}, got ${actual})"
  fi

  say "  Checksum verified (sha256)"
}

# Detect Windows (WSL, Git Bash, Cygwin) and give guidance
detect_windows() {
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) return 0 ;;
    *) return 1 ;;
  esac
}

# ============================================================
# Main
# ============================================================

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

# --- Windows check ---
if detect_windows; then
  say "Windows (MSYS2/MinGW) detected."
  say "For Windows, download the binary manually from:"
  say "  https://github.com/chihqiang/lvm/releases"
  say ""
  say "Or use one of the following methods:"
  say "  - PowerShell: Invoke-WebRequest (see docs)"
  say "  - WSL: run this script inside WSL (Linux)"
  exit 1
fi

target="$(detect_platform)"
archive_name="${BIN_NAME}-${target}.tar.gz"
archive_url="${DOWNLOAD_BASE}/${archive_name}"

tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t lvm-install)"
stage="$INSTALL_DIR/.${BIN_NAME}.$$"

# Single trap for cleanup of both temp dir and stage file
trap 'rm -rf "$tmpdir"; rm -f "$stage" 2>/dev/null || true' EXIT INT TERM

say "Installing ${BIN_NAME} for ${target}"
say "  From: ${DOWNLOAD_BASE}"
say "  To:   ${INSTALL_DIR}/${BIN_NAME}"
if [ "$VERSION" != "latest" ]; then
  say "  Version: ${VERSION}"
fi
say ""

# --- Download ---
say "Downloading ${archive_name}..."
download "$archive_url" "$tmpdir/$archive_name"

# --- Verify checksum ---
if [ "${LVM_SKIP_CHECKSUM:-}" != "1" ]; then
  verify_checksum "$tmpdir/$archive_name"
fi

# --- Extract ---
say "Extracting..."
need_cmd tar
tar xzf "$tmpdir/$archive_name" -C "$tmpdir" || fail "extraction failed (is gzip support available in tar?)"

# --- Install ---
sudo_cmd=""
resolve_sudo

$sudo_cmd cp "$tmpdir/$BIN_NAME" "$stage"
$sudo_cmd chmod 755 "$stage"
$sudo_cmd mv "$stage" "$INSTALL_DIR/$BIN_NAME"

say ""
say "Installed:"
"$INSTALL_DIR/$BIN_NAME" --version || true

# --- PATH hint ---
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    say ""
    say "Add ${INSTALL_DIR} to PATH to run ${BIN_NAME} from any terminal."
    if [ "$INSTALL_DIR" = "/usr/local/bin" ]; then
      say "  (It already is on most systems.)"
    fi
    ;;
esac

# --- Shell integration hint ---
say ""
say "Add the following to your shell config (~/.bashrc or ~/.zshrc):"
say ""
say '  eval "$(lvm env)"'
say '  eval "$(lvm hook)"'
say ""
say "Then restart your shell or run the two lines above to get started."

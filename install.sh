#!/usr/bin/env bash
set -euo pipefail

GH_REPO="chihqiang/lvm"
BIN_NAME="lvm"
INSTALL_DIR="${LVM_INSTALL_DIR:-/usr/local/bin}"
DOWNLOAD_BASE_URL="https://github.com/${GH_REPO}/releases/latest/download"

err() { printf "ERROR: %s\n" "$*" >&2; exit 1; }
info() { printf "INFO:  %s\n" "$*"; }

detect_target() {
  local os arch

  case "$(uname -s)" in
    Linux)  os="unknown-linux-musl" ;;
    Darwin) os="apple-darwin" ;;
    *)      err "unsupported OS: $(uname -s)" ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)            err "unsupported architecture: $(uname -m)" ;;
  esac

  echo "${arch}-${os}"
}

tmpdir=""
trap 'rm -rf "${tmpdir:-}"' EXIT

main() {
  local target archive_url archive_name

  target=$(detect_target)
  archive_name="${BIN_NAME}-${target}.tar.gz"
  archive_url="${DOWNLOAD_BASE_URL}/${archive_name}"

  info "target: ${target}"

  tmpdir=$(mktemp -d)

  info "downloading ${archive_url}…"
  curl -fSL --progress-bar -o "${tmpdir}/${archive_name}" "$archive_url"

  info "extracting…"
  tar xzf "${tmpdir}/${archive_name}" -C "$tmpdir"

  info "installing to ${INSTALL_DIR}/${BIN_NAME}"
  mkdir -p "$INSTALL_DIR"
  install -m 755 "${tmpdir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

  info "done — ${BIN_NAME} is ready"

  printf "\nAdd the following to your shell config (~/.bashrc or ~/.zshrc):\n\n"
  printf "  eval \"\$(lvm env)\"\n"
  printf "  eval \"\$(lvm hook)\"\n"
}

main

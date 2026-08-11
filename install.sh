#!/bin/sh
set -eu

REPO="oligamiq/codex-reset-notifier"
BIN="codex-reset-notifier"
INSTALL_DIR="${CODEX_NOTIFY_INSTALL_DIR:-$HOME/.local/bin}"
BASE_URL="https://github.com/$REPO/releases/latest/download"

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || fail "curl is required"

os=$(uname -s)
arch=$(uname -m)
case "$os:$arch" in
  Linux:x86_64|Linux:amd64) asset="$BIN-linux-x64" ;;
  Darwin:arm64|Darwin:aarch64) asset="$BIN-macos-arm64" ;;
  *) fail "unsupported platform: $os/$arch (use Cargo to build from source)" ;;
esac

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' 0 HUP INT TERM

printf 'Downloading %s...\n' "$asset"
curl --proto '=https' --tlsv1.2 -fsSL "$BASE_URL/$asset" -o "$tmp/$asset"
curl --proto '=https' --tlsv1.2 -fsSL "$BASE_URL/SHA256SUMS.txt" -o "$tmp/SHA256SUMS.txt"
expected=$(awk -v f="$asset" '$2 == f { print $1; exit }' "$tmp/SHA256SUMS.txt")
[ -n "$expected" ] || fail "checksum for $asset not found"

if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmp/$asset" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')
else
  fail "sha256sum or shasum is required"
fi
[ "$actual" = "$expected" ] || fail "SHA256 mismatch"

mkdir -p "$INSTALL_DIR"
chmod 755 "$tmp/$asset"
mv "$tmp/$asset" "$INSTALL_DIR/$BIN"

printf 'Installed %s to %s/%s\n' "$BIN" "$INSTALL_DIR" "$BIN"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) printf 'Add %s to PATH if needed.\n' "$INSTALL_DIR" ;;
esac
printf 'Next: set CODEX_NOTIFY_NTFY_TOPIC and run `%s --test-notification`.\n' "$BIN"

#!/usr/bin/env bash
# Updates the Homebrew formula with correct SHA256 hashes after a GitHub release.
# Usage: ./scripts/update-formula.sh [version]
# If version is omitted, reads from Cargo.toml.

set -euo pipefail

REPO="illobo/textStep"
FORMULA="Formula/textstep.rb"

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')}"
echo "Updating formula for v${VERSION}..."

MACOS_URL="https://github.com/${REPO}/releases/download/v${VERSION}/textstep-universal-apple-darwin.tar.gz"
LINUX_URL="https://github.com/${REPO}/releases/download/v${VERSION}/textstep-x86_64-unknown-linux-gnu.tar.gz"

echo "Downloading macOS tarball..."
MACOS_SHA=$(curl -sL "$MACOS_URL" | shasum -a 256 | cut -d' ' -f1)
echo "  SHA256: ${MACOS_SHA}"

echo "Downloading Linux tarball..."
LINUX_SHA=$(curl -sL "$LINUX_URL" | shasum -a 256 | cut -d' ' -f1)
echo "  SHA256: ${LINUX_SHA}"

# Update version
sed -i '' "s/^  version \".*\"/  version \"${VERSION}\"/" "$FORMULA"

# Update SHA256 hashes
# macOS sha is the first sha256 line, Linux is the second
awk -v macos="$MACOS_SHA" -v linux="$LINUX_SHA" '
  /on_macos/ { in_macos=1 }
  /on_linux/ { in_macos=0; in_linux=1 }
  /sha256/ && in_macos { sub(/"[^"]*"/, "\"" macos "\""); in_macos=0 }
  /sha256/ && in_linux { sub(/"[^"]*"/, "\"" linux "\""); in_linux=0 }
  { print }
' "$FORMULA" > "${FORMULA}.tmp" && mv "${FORMULA}.tmp" "$FORMULA"

echo "Formula updated:"
grep -E '(version|sha256|url)' "$FORMULA"

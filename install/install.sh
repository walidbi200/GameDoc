#!/usr/bin/env bash
set -e

REPO="walidbi200/GameDoc"
BINARY_NAME="gamedoc"
INSTALL_DIR_SYSTEM="/usr/local/bin"
INSTALL_DIR_LOCAL="$HOME/.local/bin"

VERSION="v0.1.0"
YES_FLAG=0

for arg in "$@"; do
  case $arg in
    --version)
      if [ -n "$2" ] && [[ "$2" != -* ]]; then
        VERSION="$2"
        shift 2
      else
        echo "GameDoc Installer $VERSION"
        exit 0
      fi
      ;;
    --version=*)
      VERSION="${arg#*=}"
      ;;
    --yes|-y)
      YES_FLAG=1
      ;;
    *)
      ;;
  esac
done

echo "Installing GameDoc $VERSION..."

ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
  RELEASE_ARCH="x86_64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
  RELEASE_ARCH="aarch64"
else
  echo "Error: Unsupported architecture $ARCH"
  exit 1
fi

OS=$(uname -s)
if [ "$OS" != "Linux" ]; then
  echo "Error: GameDoc only supports Linux."
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "Error: curl is required."
  exit 1
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/${BINARY_NAME}-${RELEASE_ARCH}-unknown-linux-gnu.tar.gz"
CHECKSUM_URL="https://github.com/$REPO/releases/download/$VERSION/sha256sum.txt"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$TMP_DIR"

echo "Downloading $DOWNLOAD_URL..."
if ! curl -sL "$DOWNLOAD_URL" -o "${BINARY_NAME}.tar.gz"; then
  echo "Error: Download failed."
  exit 1
fi

echo "Downloading checksums..."
if curl -sLf "$CHECKSUM_URL" -o "sha256sum.txt"; then
  echo "Verifying checksum..."
  grep "${BINARY_NAME}-${RELEASE_ARCH}-unknown-linux-gnu.tar.gz" sha256sum.txt > expected.sum
  if ! sha256sum -c expected.sum --status; then
    echo "Error: Checksum verification failed!"
    exit 1
  fi
  echo "Checksum OK."
else
  echo "No checksum file found in release. Skipping checksum verification."
fi

tar -xzf "${BINARY_NAME}.tar.gz"

if [ -w "/usr/local/bin" ]; then
  TARGET_DIR="$INSTALL_DIR_SYSTEM"
elif command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
  TARGET_DIR="$INSTALL_DIR_SYSTEM"
  USE_SUDO=1
else
  TARGET_DIR="$INSTALL_DIR_LOCAL"
fi

if [ -f "$TARGET_DIR/$BINARY_NAME" ]; then
  if [ "$YES_FLAG" -eq 0 ]; then
    read -p "GameDoc is already installed in $TARGET_DIR. Overwrite? [y/N] " response
    if [[ ! "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
      echo "Installation aborted."
      exit 0
    fi
  fi
fi

echo "Installing to $TARGET_DIR..."
mkdir -p "$TARGET_DIR"

if [ "$USE_SUDO" = "1" ]; then
  sudo mv "$BINARY_NAME" "$TARGET_DIR/"
  sudo chmod +x "$TARGET_DIR/$BINARY_NAME"
else
  mv "$BINARY_NAME" "$TARGET_DIR/"
  chmod +x "$TARGET_DIR/$BINARY_NAME"
fi

echo "GameDoc installed successfully!"
echo "Run 'gamedoc' to get started."

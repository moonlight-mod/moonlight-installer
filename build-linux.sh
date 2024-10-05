#!/usr/bin/env sh
set -e

echo "Building installer..."
cargo build --release

if [ -d ./AppDir ]; then
  echo "Cleaning up old AppDir..."
  rm -rf ./AppDir
fi

echo "Creating AppDir..."
mkdir -p ./AppDir/usr/bin
mkdir -p ./AppDir/usr/share/icons/hicolor/256x256
cp ./target/release/moonlight-installer ./AppDir/usr/bin
cp ./assets/icon.png ./AppDir/usr/share/icons/hicolor/256x256/moonlight-installer.png

if [ ! -f ./appimage-builder-x86_64.AppImage ]; then
  echo "Downloading appimage-builder..."
  # Nothing has ever gone wrong in the history of downloading random binaries off of the Internet
  wget -O ./appimage-builder-x86_64.AppImage https://github.com/AppImageCrafters/appimage-builder/releases/download/v1.1.0/appimage-builder-1.1.0-x86_64.AppImage
  chmod +x ./appimage-builder-x86_64.AppImage
fi

echo "Building AppImage..."
./appimage-builder-x86_64.AppImage --recipe ./AppImageBuilder.yml

# Move the appimage into a predictable location for CI
mv "./moonlight installer-*-x86_64.AppImage" ./moonlight-installer-x86_64.AppImage

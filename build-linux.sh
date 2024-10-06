#!/usr/bin/env sh
set -e

cargo build --release

if [ -d ./appdir ]; then
  rm -rf ./appdir
fi
mkdir appdir

cp ./assets/icon.png ./appdir
cp ./target/release/moonlight-installer ./appdir/AppRun
cp ./assets/moonlight-installer.desktop ./appdir

if [ ! -f ./appimagetool ]; then
  echo "Downloading appimagetool..."
  wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-$(uname -m).AppImage -O ./appimagetool
  chmod +x ./appimagetool
fi

./appimagetool ./appdir moonlight-installer-x86_64.AppImage

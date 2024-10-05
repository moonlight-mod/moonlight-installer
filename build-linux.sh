if [ ! -f ./appimagetool ]; then
  echo "Downloading appimagetool..."
  wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-$(uname -m).AppImage -O ./appimagetool
  chmod +x ./appimagetool
fi
export PATH=$PATH:$(pwd)

if [ -d ./target/moonlight-installer.AppDir ]; then
  rm -rf ./target/moonlight-installer.AppDir
fi

if [ -d ./target/appimage ]; then
  rm -rf ./target/appimage
fi

cargo build --release --target x86_64-unknown-linux-musl
# ???
cp ./target/x86_64-unknown-linux-musl/release/moonlight-installer ./target/release/moonlight-installer
cargo appimage --target x86_64-unknown-linux-musl

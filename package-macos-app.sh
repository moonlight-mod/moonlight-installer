#!/bin/sh

# Packaging script to build a macOS .app bundle
# If no arguments are passed, will build a single-architecture .app using the binary at
#   ./target/release/moonlight-installer
# Specifying "bundle" will combine the binaries at
#   ./target/x86_64-apple-darwin/release/moonlight-installer
#   ./target/aarch64-apple-darwin/release/moonlight-installer

APPNAME="moonlight installer.app"
EXENAME=moonlight-installer
ICON=assets/icon.icns
PLIST=assets/Info.plist

# Clear the temp folders
rm -rf temp
if [[ "$1" == "clean" ]]; then
    exit
fi

# Make our temporary folders
mkdir -p temp/app

if [[ "$1" == "bundle" ]]; then
    EXECUTABLE=temp/$EXENAME
    echo "Creating universal binary..."
    lipo -create target/x86_64-apple-darwin/release/$EXENAME target/aarch64-apple-darwin/release/$EXENAME -output $EXECUTABLE
else
    EXECUTABLE=target/release/$EXENAME
fi

# Make the app directory structure
APPDIR="temp/app/$APPNAME"
echo "Building app bundle..."
mkdir -p "$APPDIR/Contents/MacOS"
mkdir -p "$APPDIR/Contents/Resources"
# Copy our assets to it
cp $PLIST "$APPDIR/Contents/Info.plist"
cp $ICON "$APPDIR/Contents/Resources/Icon.icns"
# Copy the merged binary 
cp $EXECUTABLE "$APPDIR/Contents/MacOS/$EXENAME"

# Apply an ad-hoc signature
echo "Code signing..."
codesign --force --deep -s - "$APPDIR"

echo "Built '$APPDIR'"

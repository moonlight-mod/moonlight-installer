# moonlight-installer

A desktop application to install [moonlight](https://github.com/moonlight-mod/moonlight).

## Installation

Go to the [latest release](https://github.com/moonlight-mod/moonlight-installer/releases/latest) and download the appropriate file for your system:

- Windows users, download the `.exe`.
- macOS users with an Intel CPU, download the `x64.dmg`.
- macOS users with an Apple Silicon CPU, download the `aarch64.dmg`.
- Linux users, download the `.AppImage`.

Nightly builds on GitHub Actions can be found [here](https://github.com/moonlight-mod/moonlight-installer/actions/workflows/tauri.yml). You will need a GitHub account.

## Known issues

- There are no portable Windows builds (releases only have installers for the installer).
  - This is because Tauri requires WebView on Windows, which is only preinstalled on Windows 11.
- Linux installations are only detected if they are in `~/.local/share`.

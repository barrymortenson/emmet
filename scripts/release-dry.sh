#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# A release build without a release (plan section 17; 6A grows it into release.yml). `just release-dry [--sign]`.
#
# macOS: a universal .dmg. The emmet sidecar is built for both architectures and joined with lipo, uv comes from its
# pinned release (sha256 checked), and Tauri bundles them with tauri.release.conf.json (the sidecars and the hardened
# runtime, kept out of dev and E2E builds). With --sign: everything is signed with the Developer ID Application
# identity in the login keychain, the .dmg is notarized with `notarytool --keychain-profile emmet-notary` (H-05) and
# stapled, and spctl and stapler check it.
# Windows (CI): the unsigned NSIS installer, with the same sidecars.
#
# No secret is read or printed here: the identity and the notary profile stay in the keychain.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

sign=false
for a in "$@"; do
  case $a in
    --sign) sign=true ;;
    *) echo "release-dry: unknown option $a (use --sign)" >&2; exit 2 ;;
  esac
done

UV_VERSION=0.12.22
bins=apps/desktop/src-tauri/binaries
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$bins"

# uv for one target: the release archive, checked against its pinned sha256.
uv_for() {
  local target=$1 sha=$2 ext=$3
  local file="uv-$target.$ext"
  curl -fsSL -o "$tmp/$file" "https://github.com/astral-sh/uv/releases/download/$UV_VERSION/$file"
  local got
  # Git Bash on Windows has sha256sum, macOS has shasum.
  if command -v sha256sum >/dev/null; then got=$(sha256sum "$tmp/$file"); else got=$(shasum -a 256 "$tmp/$file"); fi
  [[ ${got%% *} == "$sha" ]] || { echo "release-dry: $file doesn't match its pinned sha256" >&2; exit 1; }
  mkdir -p "$tmp/$target"
  if [[ $ext == zip ]]; then unzip -q -o "$tmp/$file" -d "$tmp/$target"; else tar -xzf "$tmp/$file" -C "$tmp/$target"; fi
  find "$tmp/$target" -name "uv" -o -name "uv.exe" | grep -E '/uv(\.exe)?$' | head -1
}

case "$(uname -s)" in
  Darwin)
    rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
    for t in aarch64-apple-darwin x86_64-apple-darwin; do
      cargo build --release --locked -p emmet-cli --target "$t"
    done
    arm=$(uv_for aarch64-apple-darwin 5d714de09501a59393ceca78f4bc232a50478729640d251907160299b2a93ddd tar.gz)
    x86=$(uv_for x86_64-apple-darwin 1b8a5b316883df2daf20fb9a446e5b230e01d947d57aba2694977c5ac5a7e98c tar.gz)
    # Tauri's universal build compiles each architecture with its own sidecars, then bundles the universal ones.
    cp target/aarch64-apple-darwin/release/emmet "$bins/emmet-aarch64-apple-darwin"
    cp target/x86_64-apple-darwin/release/emmet "$bins/emmet-x86_64-apple-darwin"
    cp "$arm" "$bins/uv-aarch64-apple-darwin"
    cp "$x86" "$bins/uv-x86_64-apple-darwin"
    lipo -create -output "$bins/emmet-universal-apple-darwin" "$bins/emmet-aarch64-apple-darwin" "$bins/emmet-x86_64-apple-darwin"
    lipo -create -output "$bins/uv-universal-apple-darwin" "$arm" "$x86"
    if [[ $sign == true ]]; then
      identity=$(security find-identity -v -p codesigning | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -1)
      [[ -n $identity ]] || { echo "release-dry: no Developer ID Application identity in the keychain (H-05)" >&2; exit 1; }
      # The sidecars first, with the hardened runtime and a secure timestamp, as notarization requires; then Tauri
      # signs the app around them.
      for b in "$bins"/*-apple-darwin; do
        codesign --force --options runtime --timestamp --sign "$identity" "$b"
      done
      export APPLE_SIGNING_IDENTITY=$identity
    fi
    # CI=true makes Tauri's dmg step skip scripting Finder to lay the window out, which times out without a GUI
    # session (and needs Automation permission with one).
    (cd apps/desktop && CI=true pnpm tauri build --ci --target universal-apple-darwin --config src-tauri/tauri.release.conf.json --bundles app,dmg)
    dmg=$(ls -t target/universal-apple-darwin/release/bundle/dmg/*.dmg | head -1)
    if [[ $sign == true ]]; then
      codesign --force --timestamp --sign "$identity" "$dmg"
      xcrun notarytool submit "$dmg" --keychain-profile emmet-notary --wait
      xcrun stapler staple "$dmg"
      spctl --assess --type open --context context:primary-signature -v "$dmg"
      xcrun stapler validate "$dmg"
    fi
    echo "release-dry: $dmg"
    ;;
  MINGW* | MSYS* | CYGWIN*)
    [[ $sign == false ]] || { echo "release-dry: Windows builds are unsigned until Trusted Signing (plan section 17)" >&2; exit 2; }
    cargo build --release --locked -p emmet-cli
    cp target/release/emmet.exe "$bins/emmet-x86_64-pc-windows-msvc.exe"
    # Assigned first: a failing $(...) inside cp's arguments wouldn't stop the script.
    uv=$(uv_for x86_64-pc-windows-msvc ea1397797a0ca15f63516dd0f49c2dde9776db9be5861cab152ebe8ad199894d zip)
    cp "$uv" "$bins/uv-x86_64-pc-windows-msvc.exe"
    (cd apps/desktop && pnpm tauri build --ci --config src-tauri/tauri.release.conf.json --bundles nsis)
    echo "release-dry: $(ls -t target/release/bundle/nsis/*.exe | head -1)"
    ;;
  *) echo "release-dry: macOS or Windows only" >&2; exit 2 ;;
esac

#!/usr/bin/env bash
set -euo pipefail
mkdir -p release
app='build/macos/Build/Products/Release/Gosh USB Creator.app'
if [[ -n "${GOSH_MACOS_SIGNING_IDENTITY:-}" ]]; then
  codesign --force --deep --options runtime --sign "$GOSH_MACOS_SIGNING_IDENTITY" "$app"
  codesign --verify --deep --strict "$app"
fi
hdiutil create -volname 'Gosh USB Creator' -srcfolder "$app" -ov -format UDZO release/gosh-usb-creator-macos.dmg
if [[ -n "${GOSH_NOTARY_PROFILE:-}" ]]; then
  xcrun notarytool submit release/gosh-usb-creator-macos.dmg --keychain-profile "$GOSH_NOTARY_PROFILE" --wait
  xcrun stapler staple release/gosh-usb-creator-macos.dmg
fi

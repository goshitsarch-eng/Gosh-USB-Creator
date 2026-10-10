# Building and validation

Flutter 3.47.7 stable, Dart 3.13.5 and Rust stable are the tested toolchains.
The Rust dependency lockfile and root pubspec.lock are source artifacts.
Use `flutter pub get --enforce-lockfile`. The native-asset hook requires cargo on
PATH and the Rust standard library for the Flutter target architecture.

## Common checks

Run from the repository root:

```sh
flutter pub get --enforce-lockfile
dart format --output=none --set-exit-if-changed lib test integration_test packages/gosh_core/lib packages/gosh_core/hook
flutter analyze
flutter test test
cargo fmt --manifest-path packages/gosh_core/rust/Cargo.toml --check
cargo clippy --locked --manifest-path packages/gosh_core/rust/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --locked --manifest-path packages/gosh_core/rust/Cargo.toml
```

The core test uses real file-backed stream fixtures; production writes accept only
revalidated platform devices. `integration_test/core_test.dart` loads the real Rust
asset, checks Unicode, hashing, concurrent calls, failure propagation and rejection
of an arbitrary file as a target. Run it separately from unit/widget tests:

```sh
flutter test integration_test/core_test.dart -d linux
```

That command is verified on Linux; substituting macos/windows needs those actual
runners. Rust cross-target checks are useful but do not prove native app behavior.

## Linux

Install clang, cmake, ninja-build, pkg-config, libgtk-3-dev and libnotify-dev.
Run `flutter run -d linux` or build `flutter build linux --release`.
Launch `build/linux/x64/release/bundle/gosh_usb_creator` (arm64 on ARM); retain the whole bundle,
including lib and data. Package tar/DEB/RPM/AppImage outputs with:

```sh
mkdir -p release
tar -C build/linux/x64/release -czf release/gosh-usb-creator-linux-x64.tar.gz bundle
# Install rpm to enable RPM output. The AppImage tool/runtime downloads are SHA-256 checked.
tool=$(packaging/linux/fetch-appimagetool.sh /tmp/gosh-appimagetool)
APPIMAGETOOL="$tool" packaging/linux/package.sh
```

UDisks2 and an authentication agent are needed for authorization prompts. Do not
run destructive writes on host/system disks. The cloud instance uses a local
native-library prefix and `/workspace/.tools/activate.sh`; that is environment
configuration, not part of the application's production architecture.

## Windows

Install Visual Studio desktop C++ tools, Flutter and Rust MSVC. Run
`flutter build windows --release`. Inno Setup builds the installer using
`packaging/windows/installer.iss`; set GOSH_VERSION to 3.0.0 first. The complete
Release folder is required. Physical-drive writes require administrator access.
The CI job prepares this artifact; its execution and installer QA are unverified
from the Linux cloud instance. Signed Windows release configuration is not supplied.

## macOS

Install Xcode tools, CocoaPods, Flutter and Rust. Add both Rust targets for a
universal build: `rustup target add aarch64-apple-darwin x86_64-apple-darwin`.
Run `flutter build macos --release`, then `packaging/macos/package.sh` to create a
DMG. Optional GOSH_MACOS_SIGNING_IDENTITY and GOSH_NOTARY_PROFILE refer to existing
local signing/keychain configuration; never put their secret values in source.
Unsigned artifacts are explicitly development artifacts. The user-approved App Sandbox exception is applied for raw-device access;
Unix permission checks still require elevation.
Native macOS app, signing, notarization, Intel/Apple Silicon QA are unverified here.

## Flatpak

The manifest packages the real Linux release bundle, including Flutter plugins
and the native Rust asset. It consumes the release output; it is not a source
build inside the Flatpak SDK. First build the Linux release, then install signed
org.gnome.Platform//49 and org.gnome.Sdk//49 from Flathub.

```sh
mkdir -p build/flatpak
cp -a build/linux/x64/release/bundle build/flatpak/ # use arm64 on an ARM host
flatpak-builder --user --force-clean --repo=flatpak-repo flatpak-build packaging/flatpak/com.gosh.usb_creator.yml
flatpak build-bundle flatpak-repo release/gosh-usb-creator.flatpak com.gosh.usb_creator
flatpak install --user release/gosh-usb-creator.flatpak
flatpak run --user com.gosh.usb_creator
```

Finish arguments grant display, device access and UDisks2/notification bus names.
GTK_USE_PORTAL=1 is required. Source files selected through a portal must work
without a broad home/host filesystem grant. Drag/drop access may require portal
transfer; a path outside granted access must fail visibly. Real raw-device and
portal QA require a desktop session, portals, UDisks2 and disposable hardware.

The desktop workflow creates candidate artifacts on native runners and a tagged
release only after build jobs pass. Local compilation does not establish that CI
ran. See PLATFORM_SUPPORT.md for the current evidence and pending tests.

## Dependency licenses

Flutter supplies the Dart package notices automatically. Rust notices are bundled
in assets/rust_licenses.json and displayed on the same license page. After
changing Cargo.lock, regenerate and review them:

```sh
cargo metadata --locked --format-version 1 --manifest-path packages/gosh_core/rust/Cargo.toml | python3 tool/generate_rust_licenses.py
```

AppImage runtime/tool digests are pinned in the fetch script. If the upstream
continuous runtime changes, checksum verification fails; review the producer's
release metadata and deliberately update the pin. Do not skip verification.

Run Flutter builds and native integration tests sequentially. Quit an active
`flutter run` session before rebuilding or testing the same bundle, because
Flutter and native-asset hooks share output directories.

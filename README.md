# Gosh USB Creator

A Flutter desktop interface and independent Rust core for writing and verifying
ISO/IMG images on removable USB drives. Version 3.0.0 is the migration candidate.
It is **not yet a verified cross-platform production release**; see
[PLATFORM_SUPPORT.md](PLATFORM_SUPPORT.md) and [MIGRATION_AUDIT.md](MIGRATION_AUDIT.md).

Choose an image, optionally compare its SHA-256 or MD5 checksum, select a drive,
and confirm the destructive write. Read-back verification, progress, cancellation,
light/dark/system appearance, preferences, auto-eject and notifications are included.
Advanced mode shows signature details and post-write settings. Signature detection
is a heuristic and does not establish bootability or trust.

## Development

Stable Flutter 3.47.7 / Dart 3.13.5 and stable Rust are used. On Debian/Ubuntu:

```sh
sudo apt-get install clang cmake ninja-build pkg-config libgtk-3-dev libnotify-dev
flutter pub get --enforce-lockfile
flutter run -d linux
```

Flutter's native-asset hook builds and bundles Rust automatically. Node.js and a
WebView are not used by the new application. The old `src`, `src-tauri` and npm
files remain temporary reference material; their removal is pending explicit
cleanup authorization and outstanding platform QA. Do not use them for the new
app's development or releases.

See [BUILDING.md](BUILDING.md) for platform builds and validation,
[ARCHITECTURE.md](ARCHITECTURE.md) for ownership and the FFI contract, and
[the permission proposal](docs/macos-permissions-proposal.md) for the approved
macOS raw-device permission decision.

## Device permissions

Linux uses UDisks2 for authorized unmount/open/eject and can also use existing
raw-device permissions. A running UDisks2 service and desktop authentication agent
are prerequisites for authorization prompts. System disks are excluded.
Windows currently requires administrator privileges for physical drives. The macOS
App Sandbox exception was explicitly approved for raw-device access; Unix device
write permission still requires elevation. Flatpak uses native file portals and an
explicit UDisks2 system-bus permission; sandboxed startup and the native portal chooser were exercised. External file
exports remain blocked by the cloud container's denied FUSE mounts.

A write erases the selected target. Hardware testing must use a disposable USB
drive. The app does not claim that an image is bootable, safe, or from a trusted
publisher. It has no accounts, application telemetry, or cloud service dependency.

## License

AGPL-3.0; see [LICENSE](LICENSE). Existing branding and application purpose are
preserved. Third-party licenses appear in the application's license page.

# Rewrite review and next validation step

The local candidate replaces the runtime with Flutter/Dart presentation and an
independent Rust image/device/job core. Original settings can be explicitly
imported with a restorable backup. Existing image, checksum, write, verification,
theme and advanced-option behavior is implemented; cancellation, menus,
shortcuts, error displays and window persistence are added.

Linux build, native interaction, 5 Flutter tests, 2 real-native integration tests,
11 Rust tests, formatting/analyzer/Clippy, cross-target checking and Cargo audit
pass. Linux candidate packages are generated and the installed Flatpak starts,
persists settings and opens the real portal chooser. See PLATFORM_SUPPORT.md
and docs/QA.md for exact limits and unrun checks.

The candidate is committed and pushed on `rewrite/flutter-rust-desktop` in the
existing GitHub repository.
The reviewed workflow runs on `rewrite/**`, so this push starts native Windows,
macOS, Linux x86_64/ARM64 and Flatpak build/tests/package jobs. It does not push a
tag, update main, merge a PR or publish release artifacts. GitHub blocked all native jobs before starting because the owner account is
locked due to billing. Restore Actions access and rerun the workflow; any native
failures must be fixed before the cross-platform build gate can be marked passed.

Physical USB writing and full OS/installer/portal/DPI QA still need explicitly
authorized disposable hardware and the corresponding desktop environments.
The macOS entitlement exception is already approved and applied.

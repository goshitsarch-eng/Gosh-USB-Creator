# Platform support and release evidence

Version 3.0.0 is an implemented migration candidate. It is not a verified
production release. Evidence below comes from this Debian 13 x86_64 cloud
instance; native CI jobs were attempted but blocked before startup by a
GitHub account billing lock.

| Target | Implemented and configured | Actually verified here | Outstanding release gates |
|---|---|---|---|
| Linux x86_64 | Flutter embedder/plugins, Rust native asset, UDisks2/direct device adapter; tar, DEB, RPM, AppImage | Debug/release builds; real-core integration; native chooser → inspection → SHA-256; themes/navigation/resize; package generation | Disposable USB write/verify/cancel/eject; UDisks authorization; real install/uninstall on supported distributions; Wayland, DPI and notification delivery |
| Linux ARM64 | Native build and package CI job; architecture-specific Rust hook | Source/CI configuration only | Actual ARM64 runner build, bridge, packages and hardware QA |
| Windows x86_64 | Flutter runner, MSVC hook, PowerShell device discovery, volume GUID locks, Inno Setup | Rust Windows GNU target checking, including target-specific test code; this is not a linked MSVC DLL | Native build/bridge/UI tests, installer/uninstaller, administrator device operations, Unicode paths, DPI, clipboard, notifications |
| macOS | Flutter runner and native assets; diskutil/APFS mapping; approved App Sandbox exception; DMG/signing hooks; native Quit routed through cancellation | Rust Apple Silicon target checking, including APFS test code; approved entitlements parsed | Native .app/bridge/DMG, APFS/boot-volume eligibility, permissions, Command-Q during a write, signing/notarization, Retina; Intel support validation |
| Flatpak x86_64 | GNOME 49 runtime/SDK; full release bundle, portals and narrow bus permissions | Actual build, bundle, user installation, sandboxed startup, native device scan, dark-mode persistence and real GTK portal chooser opening | Portal export outside app storage is blocked by this container's denied FUSE mount; drag/drop transfer, real UDisks authorization/device operations and desktop notifications remain unverified |
| Flatpak ARM64 | Native ARM64 build/export CI job | Configuration only | Actual build/install/runtime and the same hardware/portal gates |

The Flatpak does not grant host/home filesystem access or network sharing.
`--device=all` is required for removable raw devices; UDisks2 bus access is
explicit. Real portal services and the normal nested Flatpak sandbox were used.
Rootless tools needed temporary read-only mounts at their compiled-in helper
paths and a private D-Bus session; none of the app's sandbox restrictions were
disabled. This container lacks a real system bus/UDisks2 and denies FUSE mounts.

macOS direct-access entitlements do not grant Unix device-write permission.
Windows currently requires administrator privileges. Privileged device handling
on each OS must be validated with disposable hardware before release. Adapter
source or cross-target type checking cannot establish safe platform behavior.

Linux packages preserve the complete Flutter bundle. The generated DEB/RPM
requirements include the bundle's measured minimum glibc symbol version.
AppImage still needs compatible GTK 3, libnotify, EGL and C++ runtime libraries;
it does not claim compatibility with every Linux distribution. In this container
AppImage mounted execution and Flatpak document exports require unavailable FUSE
capabilities; extracted AppImage launch can be checked separately.

See [docs/QA.md](docs/QA.md) for checks and [BUILDING.md](BUILDING.md) for exact
build/package commands. Signed Windows/macOS release credentials are not provided.
No physical device write, native CI job execution or release publication is claimed.

The native CI run is currently blocked before job startup by the GitHub owner
account billing lock. See [the validation record](docs/QA.md#native-ci-blocker).
Configured target jobs do not establish native build or test success.

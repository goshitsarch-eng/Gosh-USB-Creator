# Validation record

This records actual checks, rather than equating connected controls or configured
jobs with platform QA. The reference was audited before rewriting; see
[MIGRATION_AUDIT.md](../MIGRATION_AUDIT.md).

| Check | Outcome |
|---|---|
| Locked Flutter dependency resolution | Passed; root lockfile retained |
| Dart formatting / Flutter analyzer | Passed; analyzer rules retained |
| Flutter controller/widget suite | 5 tests passed: checksum/source races, busy guards, legacy import/backup/restore, write-start cancellation, navigation/disabled writes/errors |
| Linux native integration suite | 2 tests passed using the real Rust library: Unicode files, both hashes, concurrency/errors/fake-device rejection; UI inspection, format help, hashing, comparison and clear |
| Rust unit suite | 11 tests passed; actual multi-chunk copy/verify, corruption, write/preflight-hash cancellation, sector-aligned tails, source changes, format detection, hashing, FFI failures and target rejection; doc tests contain zero cases |
| rustfmt / Clippy all targets and features | Passed with warnings denied |
| Windows GNU / Apple Silicon Rust checking | Passed including conditional test code; no native linking/execution claimed |
| Cargo audit | Passed against 1,296 fetched RustSec advisories, 109 locked packages; advisory database is time-dependent |
| Linux release build and native launch | Passed, including extracted AppImage startup and the saved flutter run command; file chooser selected an 8 KiB MBR fixture; SHA-256 matched sha256sum |
| Linux package generation | DEB, RPM, AppImage and full-bundle tar generated; install/uninstall and distro compatibility remain unrun |
| Flatpak build/export/install | Actual GNOME 49 package built and installed per user |
| Flatpak startup/settings | Native library scan returned zero targets; dark mode persisted across launches; native widgets rendered |
| Flatpak portals | Real GTK portal chooser opened; document portal FUSE mount failed with Operation not permitted; arbitrary host-file export is not claimed |
| Windows/macOS native builds, installers and UI | Unrun: no corresponding OS environment available here |
| Disposable USB operations | Unrun: no authorized disposable hardware attached |
| Remote CI and tagged release | Configured; not executed/published |

Native screenshots in `docs/screenshots` are captured app windows, not mockups.
Xvfb provides X11 only. It cannot establish Wayland, real desktop theme watching,
Retina/high-DPI behavior or interaction with actual OS authorization agents.

## Control coverage and manual release checklist

Every visible action has an implementation. The following distinguishes current
coverage from remaining manual checks:

| Controls | Coverage and remaining checks |
|---|---|
| Write/Settings, mode, appearance, verify default | Widget/controller coverage; native navigation/theme/resize exercised; verify OS theme changes and persistence on each target |
| Image open/clear/drop, metadata and format help | Native Linux chooser plus real-core UI regression; test drag/drop and portal file grants on each desktop |
| Hash algorithm/calculate/copy/expected field | Real SHA-256/MD5/Unicode/parallel bridge tests; matching field and clear regression; manually verify clipboard with long values on every OS |
| Device refresh/selection/eject | Real empty scans; core refuses a forged file target; test hotplug, reused paths, system/boot exclusion, folder/APFS mounts and eject failures with hardware |
| Write confirmation, options, progress/cancel/retry | Connected state machine and core stream regressions; start cancellation regression; test confirmation refresh and each OS close/quit path during authorized USB writes |
| Auto-eject/notifications | Wired with visible warnings/errors; verify notification permission, actual delivery and eject outcomes on all OSes |
| Settings import/export/restore | Validation, prior snapshot and restoration regressions; verify native open/save and portal exports with real desktop storage |
| Menus/shortcuts/about/licenses | Connected commands and standard text context menus; verify Ctrl/Command conventions, native macOS menu termination and screen-reader focus |
| Window size/maximization/exit | Persistence and cancellation wiring; native Linux resize exercised; validate maximization, monitor/DPI changes and close/reopen on every OS |

Hardware QA must select an explicitly authorized disposable removable drive.
Check known-byte writes, read-back, wrong/changed checksum, insufficient capacity,
source-on-target rejection, cancellation, unplugging, failed authorization,
device identity changes and concurrent writers. Capture OS/toolchain versions and
outcomes in this record before promoting the candidate to a production release.

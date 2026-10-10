# Gosh USB Creator migration audit

## Evidence and scope

Reference: main, commit `caf87d654d8b25a31467f1f38037c0a7790840bc`.
The reference was audited before implementation. The original source tree is
retained for comparison. This record distinguishes original defects from the
implemented migration and actual validation; release gates remain open.

## Reference implementation at the audited commit

Svelte 5 / TypeScript / Vite 7 frontend, Tauri 2 desktop shell, Rust 2021 backend.
README incorrectly identifies React. This checkout already contains Linux,
Windows and macOS adapters; their existence does not establish platform support.
Entry points: `src/main.ts`, `src/App.svelte`, `src-tauri/src/main.rs`,
`src-tauri/src/lib.rs`. Six Tauri commands join presentation to backend.
`src/lib/stores/app.svelte.ts` holds shared reactive presentation state.
`src-tauri/src/commands` implements file metadata, hashing, format detection,
streamed writing and byte verification. `src-tauri/src/platform` implements
platform device discovery, unmount, open and eject. `utils/progress.rs` contains unused progress DTOs.

Dependencies: Svelte, Tauri JS API/dialog/notification, Rust Tauri plugins,
Tokio, Serde/JSON, SHA-2, MD5, thiserror and macOS plist.
The reference contained no Flutter/Dart code or bridge. No existing test suite was found.
No cancellation API, application menu, shortcut system, window-state persistence,
undo, CLI argument handling, compressed-image processing or multiboot is implemented.
Roadmap items are not parity requirements for implemented behavior.

## Data, settings and platform assumptions

Input: ISO/IMG (dialog also accepts arbitrary files). Raw bytes are copied to a
discovered device in 4 MiB blocks, flushed, optionally reopened and compared.
Hashing uses an 8 MiB buffer. ISO signature inspection at 32768, MBR at 510 and
GPT at 512 is heuristic, not full structural or bootability validation.
No project database, document format or account/network engine exists.

WebView localStorage keys: `gosh-usb-theme` (system default),
`gosh-usb-mode` (standard), `gosh-usb-verify-default` (true),
`gosh-usb-auto-eject` (false), `gosh-usb-show-notification` (true).
Migration must investigate each OS WebView storage format/location before
claiming automatic settings import; preserve old storage and provide an explicit
import path if reliable automatic reading is unavailable.

Linux: `/sys/block`, `/proc/mounts`, `/dev`, `umount`, `pkexec`, `eject`,
`udisksctl`. macOS: `diskutil` plist and `/dev/rdiskN`. Windows: PowerShell,
Get-Disk/Get-Partition/Get-Volume and Shell.Application COM verbs, PhysicalDrive.
Process execution uses argument arrays; generated PowerShell disk numbers are
parsed as integers. Synchronous process and filesystem work runs inside some
async functions. UI recommends launching the entire app as administrator.
A rewrite needs an independently constrained privileged device integration,
not privilege escalation of all presentation code.

## Feature inventory

Statuses below describe source evidence unless explicitly VERIFIED.
This table records the original source audit. Migration results follow below.

| Feature | Current state / source | Expected behavior / issue | Flutter responsibility | Rust responsibility | Platform considerations |
|---|---|---|---|---|---|
| Write/Settings navigation | PARTIAL, App.svelte | Two sidebar views; no menu or shortcuts | Navigation, focus, command wiring | None | Native keyboard conventions |
| Standard/Advanced | PARTIAL, ModeToggle, app.svelte.ts | Persist mode, reveal validation/post-write actions | Presentation and settings | None | Preserve preferences |
| Image dialog / clear | PARTIAL, ISOSelector | Select ISO/IMG, show name/size; errors console-only | Native dialog, display, clear action | File inspection | Portal-selected file access |
| Drag/drop | PARTIAL, ISOSelector handleDrop | Uses nonstandard File.path rather than Tauri native drop events; runtime unverified | Native desktop drop, URI conversion | File inspection | Verify all OSes and portal permissions |
| Device list / select | PARTIAL, devices.rs, platform/* | List removable targets with capacity and mounts | Selection and empty/error states | Enumeration and eligibility | Platform device identity and system-disk exclusion |
| Refresh / hotplug | PARTIAL, DeviceList | 8-second refresh; pause while writing, clear unplugged selection | Timer/lifecycle, busy state | Revalidate device identity | Avoid stale selection/reused paths |
| SHA-256 / MD5 | PARTIAL, verify.rs, VerificationPanel | Stream file, display hash | Algorithm selection, loading, clipboard | Streaming hashes | Unicode and large-file tests |
| Expected checksum comparison | PARTIAL, VerificationPanel | Trim/case-insensitive comparison; no input-shape validation | Input/display state | Validation where meaningful | Mismatch currently does not block write |
| Image signature / format help | PARTIAL, verify.rs, ISOSelector | ISO/MBR/GPT heuristics; unknown can be marked valid | Display explanations/warnings | Typed classification and validation | Do not imply bootability |
| Capacity validation | PARTIAL, verify.rs/write.rs | Backend checks known size; UI result can go stale on device change | Recompute on input changes | Authoritative preflight | Unknown capacity must be accounted for |
| Destructive confirmation | PARTIAL, App.svelte | Native confirmation includes device name/size | Dialog and command state | Revalidate approved target before open | Stable identity and mounted-volume exclusion |
| Stream write / sync | PARTIAL, write.rs | Copy image, sync device | Progress and operation lifecycle | Preflight, privileged write, concurrency | Whole-device exclusivity and source safety |
| Read-back verification | PARTIAL, write.rs | Reopen source/device, byte compare | Phase display | Verification and errors | Source can change between passes |
| Progress, speed, ETA | PARTIAL, ProgressBar | Event per block; listener attaches after preparing view appears | Subscribe before work, render state | Bounded progress events | Operation IDs, shutdown handling |
| Retry / write another | PARTIAL, ProgressBar | Reset terminal state | Commands/presentation | New operation lifecycle | Do not relaunch while prior work active |
| Auto-eject | PARTIAL, App.svelte/write.rs | Advanced only; eject errors console-only | Setting, result warning | Platform eject | Success must not imply ejected |
| Completion notification | PARTIAL, App.svelte | Advanced only, request permission | Established desktop plugin | Completion event | OS/portal behavior and denial |
| Verify default | PARTIAL, Settings/store | Persistent default, can change in Settings during write | Preferences, snapshot operation options | Operation-specific verify option | Persist per-user data |
| Light/Dark/System | PARTIAL, Settings/styles | CSS/localStorage | ThemeMode and system brightness | None | Native high DPI/contrast |
| About | PARTIAL, Settings | Name, version 2.0.1, AGPL-3.0 | About dialog, license | None | Retain application identity/license |
| Native packages | UNKNOWN, .github/workflows | Linux deb/rpm/AppImage, Windows/macOS jobs | Desktop embedding | Native library packaging | CI artifacts not tested here |
| Flatpak | PARTIAL, build-linux.yml | Generated manifest copies host binary; broad device/host permissions | Portals and app lifecycle | Host privileged integration | Raw device access and helper policy unresolved |

## Defects identified in source

1. `platform/macos.rs`: `disk_id.contains('s')` excludes every `diskN` identifier
   because "disk" contains `s`; whole-device discovery cannot return these disks.
2. `platform/windows.rs`: unmount command result/status is ignored; comment claims
   volume locking, but open_device_for_write merely opens a file and implements
   no explicit volume lock/dismount. Do not claim reliable Windows raw writing.
3. Linux mount matching uses starts_with(device_path), confusing e.g. sda and
   sdaa; `/proc/mounts` escape sequences are not decoded.
4. Image validation cache is cleared when the file changes, not when the target
   changes. Displayed capacity validation may be stale. Backend still checks size.
5. Async file/hash/validation results have no selection generation check;
   changing file while awaiting can apply a previous file's result.
6. Clear/select source controls remain available during write; completion text
   dereferences the current selected file, rather than immutable operation input.
7. Unmount happens before validating source metadata/size, causing unnecessary
   disruption when preflight would fail. Source/device identity can change.
8. Unknown signature with >=512 bytes yields is_valid=true with a warning;
   UI "Valid" overstates this heuristic result.
9. Device scan, file selection, hash, validation, eject and notification failures
   are often console-only; empty list can hide scan failure.
10. Progress subscription is tied to a conditionally mounted component; early
    events can be missed. No cancellation or graceful-close policy exists.
11. "You can safely remove" appears even when auto-eject was disabled or failed.
12. Svelte check: unused onMount import in ISOSelector; unassociated Calculated
    label in VerificationPanel. Production frontend build passes with warning.
13. Rust `cargo build --locked` rejects the tracked lockfile as needing update.
    Reference build is attempted in an external disposable copy, preserving it.
14. Native minimum versions claimed in README are not demonstrated by CI.

## Reference runtime and build evidence

The original frontend build passed (124 modules). Its type check failed with an
unused import and an accessibility warning. Its stale Cargo.lock required an
external disposable build copy; the original lockfile was preserved. The native
reference executable subsequently launched through PRoot with the signed local
WebKit helper directory mounted at its compiled-in path. The earlier missing
helper-path blocker was resolved before the rewrite proceeded.

On Xvfb, the original app rendered; navigation, refresh with zero devices,
Standard/Advanced, appearance and option controls and a native image chooser
were exercised. Original USB write/eject and original checksum results were not
runtime-verified. The original Rust suite ran zero tests. Source review supplied
the rest of the feature inventory, including the defects above.

## Implemented migration

Flutter/Dart owns desktop presentation, commands, preferences, window lifecycle,
native file dialogs/drop and notifications. Rust owns image inspection/hashing,
removable-device eligibility/identity, preflight, exclusive access and streaming
write/verify/cancel jobs. The direct two-symbol FFI choice, allocation ownership,
async isolation and native-asset build hook are explained in ARCHITECTURE.md.

| Reference feature | Migration and evidence |
|---|---|
| Navigation, Standard/Advanced, themes and persistent defaults | Implemented; controller/widget tests and native Linux navigation; sandboxed Flatpak dark mode persisted across launches |
| Image dialog, clear and metadata | Implemented; native Linux chooser selected a real fixture; real-core UI integration inspects a Unicode filename and clears it |
| Drag/drop | Native desktop_drop callback is wired and busy-guarded; external desktop and portal transfers still require manual QA |
| Device list, refresh and hotplug | Implemented with exact mounts, eligibility and identity revalidation; empty scans exercised on Linux/Flatpak; real hotplug unrun |
| SHA-256/MD5, clipboard and expected checksum | Implemented; both algorithms tested through real FFI; native Linux SHA-256 matched sha256sum; mismatches block write; checksum rechecked before unmount |
| Format/help and capacity | Implemented; heuristic warnings replace false validity claims; signatures/unknown tested; format-help dialog exercised by real-core UI integration |
| Confirmation, write, sync and read-back | Implemented; current approved identity checked after confirmation; authoritative core preflight; real multi-chunk file-backed copy/verification and corruption tests; physical USB QA unrun |
| Progress, retry, cancellation and close | Implemented as bounded job polling; one concurrent writer; start-in-flight cancellation regression tested; interrupted targets explicitly described as incomplete |
| Auto-eject and notification | Implemented with visible failures/warnings; options snapshotted per write; actual hardware/system notification delivery unrun |
| About, menus, shortcuts and window state | Implemented; native Linux navigation/resize/open shortcut exercised; full OS conventions and DPI/Wayland still need QA |
| Settings compatibility | Explicit legacy/new JSON import, backup, restore and export; old WebView storage untouched; schema/import/restore regressions tested |
| Packages/CI | New native build workflow and Linux DEB/RPM/AppImage/tar, Windows Inno Setup, macOS DMG, Flatpak packaging; actual evidence is in PLATFORM_SUPPORT.md |

The source defects are addressed by whole-disk macOS name parsing, checked command
statuses and held Windows volume locks, exact decoded Linux mount matching,
recomputed capacity checks, guarded selections, immutable operation inputs,
preflight before unmount, explicit unknown-format warnings, visible errors,
retained job progress and truthful eject results. Sector-aligned writes/read-back
handle non-aligned image tails, and preflight hashing cooperatively cancels. Windows/macOS behavior requires
native validation even where the Rust adapters compile for those targets.

There was no existing document database, multiboot, compressed-image support,
CLI command suite, undo history or cloud service to migrate. These remain outside
the implemented baseline rather than hidden or simulated controls.

## Settings migration and recovery

The five legacy values lived in WebView localStorage, whose backing store differs
between WebKitGTK, WebView2 and WKWebView. The new app deliberately leaves those
stores intact. In the old app's developer console, export the implemented keys:

```js
JSON.stringify(Object.fromEntries([
  'gosh-usb-theme', 'gosh-usb-mode', 'gosh-usb-verify-default',
  'gosh-usb-auto-eject', 'gosh-usb-show-notification'
].map(key => [key, localStorage.getItem(key)])))
```

Save the JSON as a file, choose Settings → Import settings in the new app, and
review the imported defaults. Unset legacy values import with the original defaults, including system appearance.
New exports use schema 1. Invalid input leaves preferences unchanged. Every
successful import saves the previous values; Restore previous settings swaps
back to that snapshot. Storage failures surface and leave the backup available.
Automatic extraction of opaque browser stores is not claimed.

## Current validation and remaining gates

See PLATFORM_SUPPORT.md and docs/QA.md for commands, counts, artifacts and limits.
Flutter 3.47.7 / Dart 3.13.5 and Rust stable 1.99.0 were used. Tests exercise the
real native library as well as controller/widget behavior. No physical target
was opened during cloud tests. Cross-compilation does not establish native OS
behavior or safe hardware writing.

The macOS App Sandbox exception was explicitly approved and applied; see
[the reviewed proposal](docs/macos-permissions-proposal.md). Native device
permission and signing/installer checks remain release gates.

Legacy source deletion was rejected by automatic approval review because the
broad destructive cleanup lacked explicit authorization. No rejected cleanup
was executed or circumvented. The retained legacy workflows are manual-only;
automatic Flutter releases use the new workflow. Cleanup remains deferred until
explicit authorization and the platform parity gates are satisfied.

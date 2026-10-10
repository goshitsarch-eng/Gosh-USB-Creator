# macOS raw-device permission decision

User explicitly approved the direct-access entitlement change; it has now been applied to both files. Native macOS validation remains outstanding.

The generated Flutter runner originally enabled `com.apple.security.app-sandbox` in both
`macos/Runner/DebugProfile.entitlements` and `macos/Runner/Release.entitlements`.
The current Rust raw-device implementation opens `/dev/rdiskN` after diskutil
unmounting. The App Sandbox does not authorize that operation even when a user
selects a disk-image source through the native file dialog.

Approved direct-access change in both files:

```diff
 <key>com.apple.security.app-sandbox</key>
-<true/>
+<false/>
```

This would permit a Developer ID distribution outside the App Sandbox. It removes
filesystem/process restrictions, and therefore expands the impact of a defect
in the application. It does not itself grant Unix device write permission;
raw disk access still needs explicit macOS authorization/elevation.
Signing, hardened runtime and notarization are separate and should remain enabled
for releases. Windows/Linux entitlements are unrelated to this proposal.

Alternative: retain the App Sandbox and implement a separately signed macOS
privileged helper with narrowly scoped disk operations and authenticated IPC.
That alternative still needs real macOS testing, provisioning and signing configuration.

Automatic approval review initially rejected applying the change without explicit
authorization. The user subsequently approved this exact change. macOS raw
writing remains unverified until tested with OS device authorization on macOS.

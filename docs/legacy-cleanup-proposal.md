# Deferred legacy cleanup

Automatic approval review rejected a prior broad source deletion because explicit
authorization was missing. None of that rejected deletion was executed.
The original implementation remains available; its four workflows are manual-only.

After native platform parity/QA gates pass, proposed removal covers exactly:

- `src/` and `src-tauri/`;
- `package.json`, `package-lock.json`, `index.html`, `svelte.config.js`,
  `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`;
- `.github/workflows/build-linux.yml`, `build-linux-arm.yml`, `build-windows.yml`,
  `build-macos.yml`.

LICENSE, CODE_OF_CONDUCT.md, branding sources, migration evidence and all Flutter,
Rust and new packaging files remain. Original icons have already been preserved
in Windows, macOS and Linux assets. Before any removal, verify a recoverable
reference archive and update documentation to match the final tree.

This proposal requests explicit cleanup authorization. Approval alone does not
declare platform/hardware QA passed or override those release gates.

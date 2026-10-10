#!/usr/bin/env bash
set -euo pipefail
dest=${1:?Provide a tool cache directory}
case "$(uname -m)" in
  x86_64) arch=x86_64; digest=ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0; runtime_digest=156f4bdbde9c52d01814600013e0a273f0118dc2de98975f3c8c63427ec79074 ;;
  aarch64) arch=aarch64; digest=f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158; runtime_digest=b4ff0030242d0c3bb12ce40541828303cf167493f4793456f0436edd6255c39d ;;
  *) exit 1 ;;
esac
mkdir -p "$dest"
dest=$(cd "$dest" && pwd)
curl -fsSL "https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-$arch.AppImage" -o "$dest/tool.AppImage"
# Digests from the producer's GitHub release asset metadata; never skip this check.
printf '%s  %s\n' "$digest" "$dest/tool.AppImage" | sha256sum -c - >&2
chmod +x "$dest/tool.AppImage"
curl -fsSL "https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-$arch" -o "$dest/runtime"
printf '%s  %s\n' "$runtime_digest" "$dest/runtime" | sha256sum -c - >&2
cd "$dest"
./tool.AppImage --appimage-extract > extract.log
printf '%s\n' "$dest/squashfs-root/AppRun"

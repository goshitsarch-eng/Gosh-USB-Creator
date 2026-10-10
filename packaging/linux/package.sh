#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
case "${GOSH_LINUX_ARCH:-$(uname -m)}" in
  x86_64|x64) deb_arch=amd64; flutter_arch=x64; image_arch=x86_64 ;;
  aarch64|arm64) deb_arch=arm64; flutter_arch=arm64; image_arch=aarch64 ;;
  *) printf 'Unsupported Linux architecture\n' >&2; exit 1 ;;
esac
bundle=$(cd "${1:-$repo/build/linux/$flutter_arch/release/bundle}" && pwd)
version=$(awk '/^version:/ {split($2, v, "+"); print v[1]}' "$repo/pubspec.yaml")
test -x "$bundle/gosh_usb_creator"
work=$(mktemp -d "${TMPDIR:-/tmp}/gosh-package.XXXXXX")
root="$work/root"
mkdir -p "$root/usr/lib/gosh-usb-creator" "$repo/release"
cp -a "$bundle/." "$root/usr/lib/gosh-usb-creator/"
install -Dm755 "$repo/packaging/linux/gosh-usb-creator-native" "$root/usr/bin/gosh-usb-creator"
install -Dm644 "$repo/packaging/linux/com.gosh.usb_creator.desktop" "$root/usr/share/applications/com.gosh.usb_creator.desktop"
install -Dm644 "$repo/packaging/linux/com.gosh.usb_creator.metainfo.xml" "$root/usr/share/metainfo/com.gosh.usb_creator.metainfo.xml"
install -Dm644 "$repo/packaging/linux/icon.png" "$root/usr/share/icons/hicolor/128x128/apps/com.gosh.usb_creator.png"
install -Dm644 "$repo/LICENSE" "$root/usr/share/doc/gosh-usb-creator/copyright"
glibc=$(find "$bundle" -type f -print0 | xargs -0 readelf --version-info 2>/dev/null | sed -n 's/.*Name: GLIBC_\([0-9.]*\).*/\1/p' | sort -V | tail -1) || true
glibc=${glibc:-2.34}
mkdir -p "$root/DEBIAN"
cat > "$root/DEBIAN/control" <<CONTROL
Package: gosh-usb-creator
Version: $version
Architecture: $deb_arch
Maintainer: Gosh USB Creator contributors <noreply@github.com>
Section: utils
Priority: optional
Depends: libc6 (>= $glibc), libgtk-3-0 (>= 3.24), libnotify4, libstdc++6, libegl1
Description: Write and verify disk images on removable USB drives
 Flutter desktop interface with an independent Rust image and device core.
CONTROL
find "$root" -type d -exec chmod 755 '{}' \;
chmod -R a+rX "$root/usr"
chmod 644 "$root/DEBIAN/control"
dpkg-deb --build --root-owner-group "$root" "$repo/release/gosh-usb-creator_${version}_${deb_arch}.deb"
if command -v rpmbuild >/dev/null; then
  rpm_top="$work/rpm"
  mkdir -p "$rpm_top/BUILD" "$rpm_top/BUILDROOT" "$rpm_top/RPMS" "$rpm_top/SOURCES" "$rpm_top/SPECS" "$rpm_top/SRPMS"
  # Packaging the already built runtime avoids a second, divergent build path.
  cat > "$rpm_top/SPECS/gosh.spec" <<SPEC
Name: gosh-usb-creator
Version: $version
Release: 1
Summary: Write and verify disk images on removable USB drives
License: AGPL-3.0-only
URL: https://github.com/goshitsarch-eng/Gosh-USB-Creator
AutoReqProv: no
Requires: glibc >= $glibc, gtk3 >= 3.24, libnotify, libstdc++, mesa-libEGL
%description
Flutter desktop interface with an independent Rust image and device core.
%install
mkdir -p %{buildroot}
cp -a "$root/usr" %{buildroot}/
%files
/usr/bin/gosh-usb-creator
/usr/lib/gosh-usb-creator
/usr/share/applications/com.gosh.usb_creator.desktop
/usr/share/metainfo/com.gosh.usb_creator.metainfo.xml
/usr/share/icons/hicolor/128x128/apps/com.gosh.usb_creator.png
%license /usr/share/doc/gosh-usb-creator/copyright
SPEC
  rpmbuild -bb --nodeps --dbpath "$work/rpmdb" --define "_topdir $rpm_top" --define "_tmppath $work" --define "_build_id_links none" --define "__os_install_post %{nil}" "$rpm_top/SPECS/gosh.spec"
  find "$rpm_top/RPMS" -name '*.rpm' -exec cp '{}' "$repo/release/" \;
fi
if [ -n "${APPIMAGETOOL:-}" ]; then
  appdir="$work/AppDir"
  mkdir -p "$appdir"
  cp -a "$root/usr" "$appdir/"
  install -m755 "$repo/packaging/linux/AppRun" "$appdir/AppRun"
  cp "$repo/packaging/linux/com.gosh.usb_creator.desktop" "$appdir/"
  cp "$repo/packaging/linux/icon.png" "$appdir/com.gosh.usb_creator.png"
  ln -s com.gosh.usb_creator.png "$appdir/.DirIcon"
  runtime=${APPIMAGE_RUNTIME:-$(dirname "$APPIMAGETOOL")/../runtime}
  test -f "$runtime"
  ARCH="$image_arch" "$APPIMAGETOOL" --runtime-file "$runtime" "$appdir" "$repo/release/gosh-usb-creator-${version}-${image_arch}.AppImage"
fi
printf 'Package staging retained at %s\n' "$work"

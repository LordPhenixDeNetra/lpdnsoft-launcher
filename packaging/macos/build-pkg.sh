#!/usr/bin/env bash
# Construit le paquet macOS du launcher (modèle aNkou, ADR 0011).
#
#   packaging/macos/build-pkg.sh <version> <arch> <binaire> <sortie.pkg>
#
# L'application est posée dans /Applications/LPDNSoft Launcher.app, la disposition que la
# résolution de chemin du launcher essaie sur macOS. Aucune signature : le paquet est refusé
# par Gatekeeper au premier lancement, ce qui est annoncé dans docs/runbooks/release.md.
set -euo pipefail

version="${1:?version manquante}"
arch="${2:?architecture manquante}"
binary="${3:?binaire manquant}"
out="${4:?sortie manquante}"

here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

app="$work/LPDNSoft Launcher.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

cp "$binary" "$app/Contents/MacOS/lpdnsoft-launcher"
chmod 755 "$app/Contents/MacOS/lpdnsoft-launcher"

sed -e "s/@VERSION@/$version/" -e "s/@ARCH@/$arch/" \
    "$here/Info.plist" > "$app/Contents/Info.plist"

mkdir -p "$work/root/Applications"
cp -R "$app" "$work/root/Applications/"

pkgbuild --root "$work/root" \
         --identifier fr.lpdnsoft.launcher \
         --version "$version" \
         --install-location / \
         "$out"

echo "paquet : $out"

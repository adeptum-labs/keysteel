#!/usr/bin/env bash
# Builds the .deb package. Run from anywhere; needs cargo-deb.
#
# The service, D-Bus and menu files keep an @BIN@ placeholder so the same
# templates serve install.sh and the package. Here it becomes /usr/bin.
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
stage=$root/target/deb-stage

rm -rf "$stage"
mkdir -p "$stage"
cp "$root/packaging/udev/70-keysteel.rules" "$stage/"
sed "s|@BIN@|/usr/bin|" "$root/packaging/systemd/keysteel-daemon.service" \
    > "$stage/keysteel-daemon.service"
sed "s|@BIN@|/usr/bin|" "$root/packaging/dbus/org.keysteel.Daemon.service" \
    > "$stage/org.keysteel.Daemon.service"
sed "s|@BIN@|/usr/bin|" "$root/packaging/keysteel.desktop" \
    > "$stage/keysteel.desktop"

cd "$root"
cargo build --release --workspace --locked
cargo deb --no-build -p keysteel-daemon
ls -1 target/debian/*.deb

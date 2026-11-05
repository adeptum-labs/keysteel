#!/usr/bin/env bash
# Installs Keysteel for the current user. Run from the repository root.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
data=${XDG_DATA_HOME:-$HOME/.local/share}
config=${XDG_CONFIG_HOME:-$HOME/.config}
cargo_root=${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}
bin=$cargo_root/bin

cargo install --path "$root/crates/keysteel-daemon" --locked --root "$cargo_root"
mkdir -p "$config/systemd/user"
sed "s|@BIN@|$bin|" "$root/packaging/systemd/keysteel-daemon.service" \
    > "$config/systemd/user/keysteel-daemon.service"
mkdir -p "$data/dbus-1/services"
sed "s|@BIN@|$bin|" "$root/packaging/dbus/org.keysteel.Daemon.service" \
    > "$data/dbus-1/services/org.keysteel.Daemon.service"

systemctl --user daemon-reload
systemctl --user enable --now keysteel-daemon.service

# The app needs the GTK 4 development files. Without them the daemon still works.
if pkg-config --exists gtk4 2>/dev/null; then
    cargo install --path "$root/crates/keysteel-app" --locked --root "$cargo_root"
    mkdir -p "$data/applications"
    sed "s|@BIN@|$bin|" "$root/packaging/keysteel.desktop" \
        > "$data/applications/keysteel.desktop"
else
    echo "Skipping the app: the GTK 4 development files were not found." >&2
    echo "Install them (libgtk-4-dev on Debian and Mint) and run this script again." >&2
fi

cat <<EOF

Keysteel is installed. One step needs root: let your user talk to the
keyboard by installing the udev rule once:

  sudo cp "$root/packaging/udev/70-keysteel.rules" /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger --subsystem-match=hidraw
EOF

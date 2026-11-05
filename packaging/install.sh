#!/usr/bin/env bash
# Installs Keysteel for the current user. Run from the repository root.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
data=${XDG_DATA_HOME:-$HOME/.local/share}
config=${XDG_CONFIG_HOME:-$HOME/.config}

cargo install --path "$root/crates/keysteel-daemon" --locked
if [ -d "$root/crates/keysteel-app" ]; then
    cargo install --path "$root/crates/keysteel-app" --locked
    install -Dm644 "$root/packaging/keysteel.desktop" "$data/applications/keysteel.desktop"
fi

install -Dm644 "$root/packaging/systemd/keysteel-daemon.service" \
    "$config/systemd/user/keysteel-daemon.service"
mkdir -p "$data/dbus-1/services"
sed "s|@HOME@|$HOME|" "$root/packaging/dbus/org.keysteel.Daemon.service" \
    > "$data/dbus-1/services/org.keysteel.Daemon.service"

systemctl --user daemon-reload
systemctl --user enable --now keysteel-daemon.service

cat <<EOF

Keysteel is installed. One step needs root: let your user talk to the
keyboard by installing the udev rule once:

  sudo cp "$root/packaging/udev/70-keysteel.rules" /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger --subsystem-match=hidraw
EOF

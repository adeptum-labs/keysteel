# Keysteel

An unofficial Linux companion for SteelSeries keyboards, in the spirit of
SteelSeries GG. Keysteel runs quietly in the background from the moment you
log in to your desktop and lets you configure your keyboard without Windows.

The first target device is the **Apex Pro TKL** with its OLED display.

> Keysteel is an independent project. It is not affiliated with, endorsed by
> or supported by SteelSeries. SteelSeries and Apex are trademarks of their
> respective owners.

## Status

Early days. The daemon can show scenes on the OLED of the Apex Pro TKL
Gen 3 (a Boing ball and Mario running sideways, or nothing), and a desktop app
picks the scene. The feature list below is what Keysteel is meant to do, not
all of what it does today.

## Planned features

- Per-key RGB lighting and lighting effects
- Key remapping
- Macros
- Profiles that can be switched on the fly
- OLED display content (text, images, status information)

## Planned design

- Written in Rust.
- **Background service** that talks to the keyboard over USB HID and applies
  the active profile. It starts at login through a systemd user service or an
  XDG autostart entry and is built to idle on a few megabytes of memory.
- **Desktop app** with a tray icon, opened on demand, that configures the
  service.
- A udev rule so the keyboard can be used without root privileges.
- Primary desktop target: Linux Mint (Cinnamon).

## Building

You need a recent stable Rust toolchain, plus the development files for
libudev and (for the app) GTK 4.

```
packaging/install.sh
```

The script installs the daemon for your user and starts it as a systemd user
service. It installs the app too when the GTK 4 development files are found,
and then prints the one command that needs root: installing the udev rule that
lets your user talk to the keyboard.

Start Keysteel from the application menu and pick a scene in the window. The
daemon has to be running; the window says so and offers Retry if it is not.
You can also pick a scene from a terminal:

```
busctl --user call org.keysteel.Daemon /org/keysteel/Daemon \
    org.keysteel.Daemon SetScene s boing
```

The scene ids are `off`, `boing` and `mario`. `off` gives the screen back to
the keyboard's own display.

The roller in the corner of the keyboard changes the volume. Tick "Roller
scrolls the window" in the app to make it scroll like a mouse wheel
instead, or run `busctl --user call org.keysteel.Daemon
/org/keysteel/Daemon org.keysteel.Daemon SetRollerScroll b true`. The
setting is remembered. It needs the same udev rule as the display, which
also lets your user open the keyboard's media key device and `/dev/uinput`.

By default the keyboard draws its own status text over the top 12 rows of
the OLED. Turn off "display icons" in the keyboard's OLED menu to use the
whole 128x40 screen. Keysteel cannot change that option: it is only
available from the keyboard.

## License

Copyright © 2026 Adeptum AB and Adam Waldenberg.

Keysteel is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option)
any later version. See the [LICENSE](LICENSE) file for the full text.

# Keysteel

An unofficial Linux companion for SteelSeries keyboards, in the spirit of
SteelSeries GG. Keysteel runs quietly in the background from the moment you
log in to your desktop and lets you configure your keyboard without Windows.

The first target device is the **Apex Pro TKL** with its OLED display.

> Keysteel is an independent project. It is not affiliated with, endorsed by
> or supported by SteelSeries. SteelSeries and Apex are trademarks of their
> respective owners.

## What it does

- Shows scenes on the OLED display of the Apex Pro TKL Gen 3: an Amiga
  Boing ball and Super Mario running, jumping over pipes and onto brick
  platforms in a level that never repeats, and the Adeptum logo scrolling
  across the screen. "Off" gives the
  screen back to the keyboard's own display.
- Turns the roller in the corner into a mouse wheel, with a speed you can
  set.
- Lights the keys from presets: a very faint warm yellow sheen, one solid
  colour, a slow breathing, all dark, or the keyboard's own scheme. The
  intensity, colour and breathing speed are adjustable.
- Picks the scene, the lighting and the roller settings from a small
  desktop app.

## How it is built

- Written in Rust.
- A **background daemon** owns the keyboard. It draws the scenes, handles
  the roller and offers a D-Bus service, `org.keysteel.Daemon`, on the
  session bus. It starts at login as a systemd user service, or on demand
  through D-Bus activation, and sleeps when nothing is animating.
- A **desktop app** (GTK 4), started from the application menu, talks to
  the daemon over D-Bus. It has no tray icon.
- A udev rule lets your user use the keyboard without root.
- Primary desktop target: Linux Mint (Cinnamon).

## Building

You need a recent stable Rust toolchain, plus the development files for
libudev and (for the app) GTK 4.10 or newer.

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

The lighting is set the same way. The arguments are the scheme (`warm`,
`solid`, `breathing`, `off` or `default`), the intensity from 0 to 100, red,
green and blue, and the breathing speed from 1 to 10:

```
busctl --user call org.keysteel.Daemon /org/keysteel/Daemon \
    org.keysteel.Daemon SetLighting syyyyy warm 30 255 255 255 4
```

The lighting stays on while the daemon runs; stop the daemon and the
keyboard goes back to its own lighting.

The scene ids are `off`, `boing`, `mario` and `adeptum`. `off` gives the screen back to
the keyboard's own display.

The roller in the corner of the keyboard changes the volume. Tick "Roller
scrolls the window" in the app to make it scroll like a mouse wheel
instead, or run `busctl --user call org.keysteel.Daemon
/org/keysteel/Daemon org.keysteel.Daemon SetRollerScroll b true`. The
scroll speed slider sets how far one click goes (1 to 10 notches, 3 by
default; `SetRollerSpeed u 5`). Both settings are remembered. It needs
the same udev rule as the display, which also lets your user open the
keyboard's media key device and `/dev/uinput`.

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

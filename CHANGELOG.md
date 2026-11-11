# Changelog

## 0.1.2

- New display scene that shows the total CPU load and a matrix with one
  cell per active core, filled by how busy the core is.
- The text in the Adeptum logo is larger, with evenly drawn letters.

## 0.1.1

- Fixed the key lighting stopping after a replug or after the keyboard
  refused a report. Reports to the keyboard now take turns and are tried
  again, the display and the lighting are reattached separately, and
  changing a setting picks up a keyboard that was dropped.
- The warm sheen is now a more amber colour.
- Source files carry licence headers.

## 0.1.0

First release, for the Apex Pro TKL Gen 3 over USB.

- OLED display scenes: Amiga Boing ball, Super Mario running over pipes and
  brick platforms in a level that never repeats, and the Adeptum logo
  scrolling across. "Off" gives the screen back to the keyboard.
- The roller in the corner can scroll like a mouse wheel, with an adjustable
  speed.
- Key lighting presets: a very faint warm yellow sheen, a solid colour, a slow
  breathing, all dark, or the keyboard's own scheme, each with adjustable
  intensity (and colour and speed where they apply).
- A desktop app to choose all of the above, and a background daemon that
  keeps it applied and reattaches after a replug.

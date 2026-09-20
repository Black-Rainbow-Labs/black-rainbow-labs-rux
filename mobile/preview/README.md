# `mobile/preview` — the phone, on this machine

```powershell
cargo run --release -p rux-mobile-preview
cargo run --release -p rux-mobile-preview -- --device iphone-se --theme light-neumorphic
cargo run --release -p rux-mobile-preview -- --screen gallery
cargo run --release -p rux-mobile-preview -- --wide          # 1200 points: the expanded size class
cargo run --release -p rux-mobile-preview -- --list
```

It runs `mobile/app` inside a phone frame at a named device's **logical** size, with that device's
published safe area insets and a touch pointer forced, and prints underneath what it is showing: the
device, the size, the size class, the insets and the minimum target.

| | Points | Insets, top / bottom |
|---|---|---|
| iPhone 16 | 393 x 852 | 59 / 34 |
| iPhone 16 Pro Max | 440 x 956 | 62 / 34 |
| iPhone SE | 375 x 667 | 20 / 0 |
| Pixel 8 | 412 x 870 | 24 / 24 |

## What it is for, and what it is not

The Android emulator is the real thing and it runs on this machine. What it cannot show is an iPhone's
Dynamic Island inset, and no emulator here can run iOS at all — so this is where an iPhone's geometry
gets checked. The two inset lines are drawn over the application in amber, so a control that strays into
the notch or the home indicator is visible rather than merely wrong.

It is a preview of the **layout**, not of the platform. There is no status bar, no keyboard and no system
gesture. What it does show is exactly what the application does with the width and the insets it is
given, which is what nearly every mobile layout mistake is.

`--background`, `--device`, `--screen` and `--theme` reach any state from the command line, so
`tools/capture-mobile.ps1` photographs all thirty frames with no synthetic input at all — which matters
on Windows, where activating a window switches the virtual desktop with it.

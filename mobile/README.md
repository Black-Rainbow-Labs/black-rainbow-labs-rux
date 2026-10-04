# `mobile/` — rux on a phone

An Unluminous mobile application is coming and it will be built out of this library. This folder is the
working example of how: one application crate with no platform code in it, and three shells that turn it
into a program.

```
mobile/
  app/            the application — the screens, built from rux components
  ios/            the iOS shell: a bin, a staticlib, Info.plist, and the build that makes an .app
  android/        the Android shell: a cdylib, a Gradle project, and the build that installs it
  android/shell/  `rux-android`: insets, the soft keyboard and the back gesture for any rux app on Android
  android/rux-shell/  the Kotlin half of that: `RuxActivity` (a GameActivity) and `RuxBridge`
  preview/        a desktop program that runs the application inside a phone frame, on this machine
```

`tasks/task-2032-rux-mobile-tdd.md` is the design.

## Which of these actually runs where

| | builds on Windows | runs on Windows | runs on a device |
|---|---|---|---|
| `preview` | ✅ | ✅ | — |
| `android` | ✅ | ✅ on the emulator | ✅ |
| `ios` | ✅ **including the link and the signature** | ❌ there is no iOS Simulator for Windows | needs a device and a way to install |

**The iOS build is complete on this machine.** `pwsh mobile/ios/build-ios.ps1` compiles for
`aarch64-apple-ios`, links a real arm64 Mach-O against the iPhoneOS SDK's stubs, writes the `.app` and
signs it — no Mac anywhere. What it cannot do is install it and run it, because that needs a phone.

## Start here

```powershell
# the application on this machine, at a real phone's size with that phone's safe area
cargo run --release -p rux-mobile-preview
cargo run --release -p rux-mobile-preview -- --device iphone-se --theme light-neumorphic
cargo run --release -p rux-mobile-preview -- --list

# Android: build, install on whatever is attached, launch it, photograph it
pwsh mobile/android/build-android.ps1 -Capture _agent_output/android.png

# iOS: compile, link, bundle, sign
pwsh mobile/ios/build-ios.ps1

# every screen on every device in both themes, with no synthetic input at all
pwsh tools/capture-mobile.ps1 -Wide
```

## The one thing the application does that the storybook does not

It tells `rux` what kind of device it is on, once a frame, before anything draws:

```rust
state.set_screen(Screen::phone(size, insets));
```

Everything else follows from that one call — every touch target at least 44 points, a dialog that comes
up from the bottom edge instead of the middle, a nav strip that fits 320 points, and no hover state
anywhere. Nothing in `mobile/app` asks what platform it is on.

## Why there is no second renderer

`eframe` 0.36 supports iOS and Android already, which is not what the search results say and is what the
crate source says. `egui-winit` has a `safe_area.rs` that reads `UIWindow.safeAreaInsets` through
`objc2-ui-kit`; `screen_size_in_pixels` uses `outer_size` on iOS because `inner_size` excludes the
Dynamic Island; `eframe` has `android-native-activity` and `NativeOptions::android_app`. So the phone
runs the same `eframe::App` the storybook runs, over the same `rux`.

## The crate types, and why each shell is its own crate

Cargo builds **every** crate type in a library's `crate-type` list whenever that library is built. With
`["staticlib", "cdylib", "rlib"]` on the application crate, an iOS build that wanted a bin and an rlib
also tried to link `librux_mobile.dylib` — and a dylib is a link, and a link wants every Apple framework
resolved. So:

| Crate | Crate type | Loaded by |
|---|---|---|
| `mobile/app` | `rlib` | the three shells |
| `mobile/android/glue` | `cdylib` named `rux_mobile` | `GameActivity` (`MainActivity`), via `android.app.lib_name` |
| `mobile/ios` | `staticlib` named `rux_mobile`, and a `bin` called `RuxMobile` | Xcode, or the bundle |
| `mobile/preview` | `bin` | nothing: it runs here |

## What is deliberately missing

- **Nothing about the keyboard on iOS beyond what winit does.** winit's own `WinitView` implements
  `UIKeyInput` and becomes first responder when egui asks for text, so typing works; the field is not yet
  moved above the keyboard there, because iOS reports the keyboard's height only through notifications.
  On Android `rux-android` reports the height and the example app treats it as bottom inset (task-2189).
- **A tablet layout.** `SizeClass::Medium` and `Expanded` are defined and reported, and past the compact
  breakpoint the application keeps a readable column and centres it rather than stretching. A layout that
  genuinely uses a tablet's width is a different design.

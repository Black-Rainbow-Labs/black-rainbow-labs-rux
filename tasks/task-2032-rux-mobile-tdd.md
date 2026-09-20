# rux on a phone — technical design

**task-2032** · September 2026 · the repository is `black-rainbow-labs-rux`, and `tasks/task-2008-rux-tdd.md`
is the design this one extends.

An Unluminous mobile application for iOS and Android is coming, and it will be built out of this
library. This document is about the two separate claims that has to make good on:

1. **rux works well on a phone.** It compiles for one already — the library depends on `egui`,
   `vello_cpu` and `kurbo` and on nothing platform specific. Working *well* is a different claim, and
   §4 is the list of places where it does not yet.
2. **There is a mobile application in the repository** that somebody can read and copy, for iOS and for
   Android.

§2 answers the second one by measurement. The ticket's premise was that task-1995's macOS toolchain also
gets us iOS; it gets us four of the five steps, and the fifth needs a phone.

---

## 1. What is being built

```
mobile/
  app/            the application: three screens built from rux, one crate, no platform code
  ios/            a bin and a staticlib, Info.plist, the linker shim, and build-ios.ps1
  android/        a cdylib, a Gradle project, and build-android.ps1
  preview/        a desktop program that runs the application inside a phone frame, on this machine
```

and, in the library:

```
crates/rux/src/screen.rs      Screen, SizeClass, Pointer, Insets — what a component asks about the device
```

plus the changes to the components in §5.

Everything in the library is **additive and defaults to what it does today**, so every existing test
keeps passing unchanged. A `Screen::desktop` is a mouse with no insets and no minimum target, which is
exactly the behaviour of the code before this ticket.

---

## 2. What this machine can and cannot do for iOS

> **This section was written before the SDK existed on this machine, and then rewritten once it did.**
> The first version said the link could not happen here. It can, and what follows is what was measured
> rather than what was expected. The history is left in §2.5 because the reasoning that turned out to be
> wrong is the same reasoning somebody will apply to the next Apple platform.

Four things have to work to put an application on an iPhone, and on this Windows box they have four
different answers. Each one was run rather than reasoned about.

| | Answer | How it was established |
|---|---|---|
| **Compile** the Rust for `aarch64-apple-ios` | ✅ and it needs no Apple SDK at all | `cargo check --target aarch64-apple-ios` over the whole `eframe` + `wgpu` + `vello_cpu` graph finished in **30 seconds**, including `objc2-ui-kit`, `objc2-metal`, `raw-window-metal`, `wgpu-core-deps-apple` and `arboard`. A `staticlib` is an archive and nothing in it is resolved |
| **Link** an iOS executable | ✅ with an iPhoneOS SDK | `zig cc -target aarch64-ios -isysroot <sdk>` produces a **14.1 MB arm64 Mach-O** with `LC_BUILD_VERSION platform=2 (iOS)`, linking UIKit, Metal, QuartzCore, CoreGraphics, Foundation and `libobjc` |
| **Bundle and sign** | ✅ | an `.app` for iOS is a flat directory with an `Info.plist`; `rcodesign` writes the signature and `_CodeSignature/CodeResources`, and the result carries `LC_CODE_SIGNATURE` with 120 KB of signature in it |
| **Install and run** | ❌ not here, by anyone | there is no iOS Simulator for Windows, and putting a build on a device needs Apple's own tooling plus a device |

So `pwsh mobile/ios/build-ios.ps1` produces a signed iOS application that has never been near a Mac. The
one step left is a phone.

### 2.1 The SDK

**iPhoneOS 16.5**, at `J:/mac-sdk/theos-sdks/iPhoneOS16.5.sdk` with a junction at
`J:/mac-sdk/iPhoneOS.sdk` so it sits beside `MacOSX.sdk` under the name the build script looks for. It
came from [`theos/sdks`](https://github.com/theos/sdks), sparse-cloned so only that one of the eight came
down — 260 MB. It holds `UIKit.tbd` targeting `arm64-ios`, plus Metal, QuartzCore, Foundation,
CoreGraphics, AudioToolbox and the `libSystem` stubs.

It is Apple's SDK redistributed by somebody else, which is the same licence question `MacOSX.sdk` raised
on task-1995 and which Jason settled the same way. It is not in the repository, and `build-ios.ps1` says
which three paths it looked in when there is none.

### 2.2 The linker, which is where all the work was

`rustc` drives a C compiler as its linker, and there is not one on Windows that emits Mach-O. zig is
that compiler, and `mobile/ios/zig-ios-cc` stands between the two. Five things stopped the build dead,
each one until it was found:

1. **rustc passes a response file, not arguments.** A linker command line this long does not fit on
   Windows, so rustc writes `@<path>` and the file holds several hundred arguments. The first version of
   the shim filtered its own `argv`, found two entries, and filtered nothing.
2. **`-target arm64-apple-ios10.0.0`** — rustc names the architecture the way Apple does and zig names
   it the way LLVM does, so zig answers `unknown architecture: 'arm64'` and prints its whole list.
3. **`-exported_symbols_list`** — `unsupported linker arg`. It hides a library's private symbols, which
   matters for a `cdylib` and not at all for an executable.
4. **`-isysroot` alone is not enough** — zig answers `unable to find libSystem system library` without
   an explicit `-L` into the SDK's `usr/lib`. That one flag is what stood between this box and an iOS
   binary.
5. **zig reserves no header room for the code signature**, the way Apple's linker does, so `rcodesign`
   stopped with `insufficient room to write code signature load command` —
   `-Wl,-headerpad_max_install_names` fixes it. A linker flag's problem wearing a code signing error's
   clothes.

It is a Rust program rather than a `.cmd` for two reasons, both measured: `cmd` mangles a command line
that long, and a batch file cannot drop an argument from a list.

### 2.3 Getting it onto a phone, which is the part that is not done

Two routes, and both need something only Jason can supply.

[`xtool`](https://github.com/xtool-org/xtool) builds, signs, installs and launches iOS applications from
Linux and WSL with no Xcode and no Mac. Its prerequisites: a Swift 6.3 toolchain in WSL, `usbmuxd` with
USB passthrough via USBIPD, **a physical iPhone on the cable**, an Apple ID, and the Darwin SDK extracted
from an Xcode 26 download. `mobile/ios/xtool.yml` is the configuration it reads.

Or a Mac: open the project, or drop `librux_mobile.a` into one that exists, press Run.

Nothing here logs into an Apple ID on its own initiative — the task-1898 rule — and nothing here signs
with Jason's certificate unless `-P12` names one. The default is an ad hoc signature, which is what a
build with no certificate gets.

### 2.4 Android, which launches here today

Checked piece by piece rather than assumed: **NDK 27.1.12297006** (and 27.0.12077973), platforms
28/33/35/36, build-tools to 36.0.0, `platform-tools/adb`, the emulator with two AVDs
(`Medium_Phone`, `GuitarTuner_API_36`), and Android Studio's bundled JBR for Gradle.
`aarch64-linux-android` compiles the whole graph once `eframe`'s `android-native-activity` feature is
on, and `cargo-ndk` supplies the NDK's clang as the linker.

The ticket says iOS is preferred and Android is what to do if iOS will not launch. Both are built:
**iOS gets the complete project and a compile-verified binary; Android gets the one that installs and
runs.**

### 2.5 What the macOS toolchain did and did not carry over

The ticket's premise was that task-1995's macOS work also gets us iOS. **The compile and the Mach-O both
carried over; the SDK did not, and neither did the run.** task-1995 recorded exactly the same shape for
Unluminous: `unluminous-cli` cross compiles with no SDK because it links no framework, and the windowed
build needed one copied directory before it would link. iOS was that situation again with a different
directory — a directory this box did not have until this ticket downloaded one.

The part that genuinely does not carry over is the run. macOS builds could at least be checked by Jason
on his MacBook; an iOS build cannot be checked by anything on this network without a phone.

---

## 3. `eframe` 0.36 already supports both platforms

This is the finding that decided the architecture, and it is not what a search says — it is what the
crate source says.

- `egui-winit` 0.36.2 has **`src/safe_area.rs`**: it reads `UIWindow.safeAreaInsets` through
  `objc2-ui-kit` and feeds it to egui as `RawInput::safe_area_insets`, refreshed on `Resized`,
  `ScaleFactorChanged`, `Focused(true)` and `Occluded(false)`. egui exposes it as
  `InputState::safe_area_insets()`, and `Rect - SafeAreaInsets` is implemented.
- `screen_size_in_pixels` uses `outer_size` on iOS, with the comment that `inner_size` excludes the
  area behind the Dynamic Island.
- `run_native` has no `cfg` excluding iOS, and `native/run.rs` carries `#[cfg(not(target_os = "ios"))]`
  arms around the parts that do not apply — the pump-events API and the window geometry.
- `eframe` has `android-native-activity` and `android-game-activity` features and
  `NativeOptions::android_app`, and it `compile_error!`s the one combination that does not work
  (`accesskit` with `android-native-activity`).

**So there is no second renderer and no hand-written shell.** The `eframe::App` the storybook runs is
what a phone runs, and `mobile/app` is a library crate that both shells and the desktop preview load.

The one thing the shells differ on is the entry point, which is what each platform demands:

| | Entry | Crate type |
|---|---|---|
| iOS | `#[no_mangle] extern "C" fn rux_mobile_main()`, called from `main.m`; winit's iOS backend calls `UIApplicationMain` inside `run_native` | `staticlib` (and a `bin` for the SDK-present path) |
| Android | `#[no_mangle] fn android_main(app: AndroidApp)`, which `android-activity` declares as `NativeActivity`'s entry | `cdylib` named `main` |
| Preview | an ordinary `fn main()` | `bin` |

---

## 4. Where rux is not yet good on a phone

Read out of the code rather than guessed at.

**1. Three of the five icon button sizes are too small to tap.** `IconButtonSize` draws at 24, 28, 32,
40 and 48 points. Apple's guidance is 44×44 and Material's is 48×48. The 24pt panel collapse handle is
half a legal target.

**2. `interact` claims exactly the drawn shape.** `IconButton::show` deliberately shrinks the rect it
is given to the circle's own square, "because a circle drawn into an oblong is an oval" — right for the
drawing, and it means the pressable area is the circle.

**3. The page is three fixed columns.** `examples/pages/image_creator.rs` is a 260pt panel, a canvas
and a 320pt panel. At 390 points wide that is not a layout.

**4. A dropdown anchored under its trigger and a dialog centred on the window are desktop idioms.**
`Select` opens a menu of the trigger's own width, flipping upwards when it would fall off the bottom;
`Modal` centres `min(920, 100vw − 48)`. Both *work* at 390 points and neither is what a phone does,
which is to come up from the bottom edge at full width.

**5. Nothing in the library knows about a safe area.** The Dynamic Island covers the top 59 points of
an iPhone 16 and the home indicator the bottom 34. egui hands us both and no component reads them.

**6. Nineteen rules in the reference change on `:hover`.** A touch pointer has no hover state, and
egui keeps a touch's last position after the finger lifts, so the wash stays behind on the control that
was tapped — the usual symptom of a desktop interface run on a phone.

**7. The scrollbar is 8 points wide with a 6 point thumb**, and it is a drag target.

---

## 5. The design

### 5.1 `Screen` — the one thing a component asks about the device

```rust
pub enum Pointer { Mouse, Touch }

pub enum SizeClass { Compact, Medium, Expanded }   // < 600, 600..840, >= 840 points wide

pub struct Insets { pub top: f32, pub right: f32, pub bottom: f32, pub left: f32 }

pub struct Screen {
    pub size: Vec2,
    pub insets: Insets,
    pub pointer: Pointer,
}

impl Screen {
    pub fn desktop(size: Vec2) -> Self;           // Mouse, no insets — today's behaviour
    pub fn phone(size: Vec2, insets: Insets) -> Self;
    pub fn read(ctx: &egui::Context) -> Self;     // content_rect + safe_area_insets + any_touches
    pub fn class(&self) -> SizeClass;
    pub fn is_compact(&self) -> bool;
    pub fn min_target(&self) -> f32;              // 0.0 on a mouse, 44.0 on a touch
    pub fn safe(&self, rect: Rect) -> Rect;       // the rect with the insets taken off
}
```

The breakpoints are Material's window size classes, because they are the published numbers the rest of
the industry lays out against and because 600 is the tablet line in practice. `min_target` is 44 rather
than 48 — Apple's number, and the smaller of the two, chosen because it is the one that keeps the
reference's own 40pt `.toolbtn` closest to the size it was drawn at.

It lives on `RuxState` behind a `Cell`, beside the theme:

```rust
rux.state.set_screen(Screen::read(ctx));   // once a frame, by the application
rux.screen()                               // read by a component
```

**Behind a `Cell` on the state rather than a field on `Rux`**, for the same reason the theme is: `Rux`
is constructed inside `render::layer` and inside `Rux::overlay`, so a field would have to be threaded
through every call site including the ones a consumer writes, and a popup opens a nested layer while
its opener is still drawing. The state is already the thing that outlives a frame.

### 5.2 A touch target grows the layout box, not the hit box

The rule, and it is the one decision in this document worth arguing about:

> **In touch, a control's `measure()` grows to `min_target`. What is drawn does not change.**

A 28pt circle keeps being a 28pt circle and sits centred in a 44pt box.

The alternative — leave the layout alone and quietly expand the rectangle handed to `interact` — is
what a first attempt reaches for, and it is wrong: two 28pt icon buttons 6 points apart would claim
44pt targets that **overlap by 10 points**, and in egui the later widget wins the hover, so the second
button silently eats a strip of the first one's area. Growing the layout cannot do that, because the
layout is what keeps controls apart.

`IconButton::show` still needs the hit rectangle to be the outer box rather than the circle, so it
gains one line: the circle is drawn at `side`, and `interact` is given
`side.max(screen.min_target())` centred on the same point. On a mouse `min_target()` is `0.0` and the
expression is the old one exactly.

A caller that hard-codes a 28pt box on a phone instead of asking `measure()` gets overlapping targets,
and that is documented on the method rather than defended against — the mobile screens place with
`measure()`.

### 5.3 What each component does differently

Every one of these is `screen`-conditional, so desktop is untouched.

| Component | On a touch pointer | On a compact width |
|---|---|---|
| `Button` | height at least 44 | — |
| `IconButton` | footprint at least 44, circle unchanged | — |
| `Select` | trigger and menu rows at least 44 | the menu is a **sheet** from the bottom edge, full width, with the bottom inset as padding |
| `Modal` | close button already 32 → 44 | a **sheet**: full width less a 12pt margin, bottom anchored, only the top corners rounded, a 36×4 grabber, bottom inset added |
| `TextInput`, `TextArea` | at least 44 | — |
| `Stepper` | the two hit halves at least 44 | — |
| `Switch`, `DiceToggle`, `Segmented` | at least 44 | — |
| `Chip` | at least 44 | — |
| `ListItem`, `menu_heading` | row at least 44 | — |
| `TopNav` | item height at least 44 | **fits the width it is given**: it drops the active item's label when the strip would overflow, and then trims item padding to a 10pt floor |
| `Scrollbar` | 14pt track, 10pt thumb | — |
| `Panel` | — | the header's collapse handle grows; nothing else changes |

`TopNav` is the only one that changes shape from width alone, and it does it by walking back the same
two rules that make the strip breathe on a desktop — `.topnav__item.is-active span { display: inline }`
first, then `padding: 9px 18px`. Dropping the label is the reference's own idle state, so the compact
nav is a state the design already has rather than a new drawing.

### 5.4 A sheet is a `Modal` with a different frame

Rather than a second component, `Modal` gains a presentation:

```rust
pub enum Present { Dialog, Sheet }
Modal::new("Prompt Manager").present(Present::Sheet)     // or
Modal::new("Prompt Manager")                             // Sheet automatically when compact
```

The frame, the scrim, the header, the body and the footer are the same code and the same tokens; what
changes is the rectangle, the corner set and the grabber. `Select`'s compact menu uses the same
geometry, which is why the two look like one idea on a phone rather than two.

### 5.5 Hover, and the wash left behind

`Visual::of` gains the pointer: on `Pointer::Touch` there is no `Hover` state at all, only `Idle`,
`Active` and `Disabled`. That is one line in one place and it removes the whole class of stuck washes,
because nothing downstream has a hover to draw.

### 5.6 Scrolling

egui already turns a single-finger drag inside a `ScrollArea` into a scroll with inertia, so the work
here is not to implement scrolling — it is to make sure the mobile screens put their content in a
`ScrollArea` and that `Select`'s own menu keeps reading `smooth_scroll_delta`, which already works from
a drag as well as a wheel.

---

## 6. `mobile/app` — the application

One crate, `rux-mobile-app`, holding an `eframe::App` and three screens. It has no platform code in it
at all; each shell is a dozen lines that construct it.

**Screen 1 — Create.** The Image Creator laid out for a phone: the top nav, the project menu, the
canvas as a square tile, the prompt as a growing text area with its character count, and the controls
in a scrolling column of collapsible groups. The model picker and the dimension picker open as sheets.
This is the same components as `examples/pages/image_creator.rs` and none of the same layout, which is
the point being demonstrated.

**Screen 2 — Gallery.** A two-column `repeat(auto-fill, minmax())` grid of tiles, to show the grid
solver behaving at 390 points and to give the scroll something to do.

**Screen 3 — Components.** A short gallery of every control at touch size, so the phone can be used to
check the library the way the storybook is used on a desktop. It is the same story list idea, not the
same code — the storybook's sidebar and filter are a desktop shape.

The three are reached from `TopNav` at the top, in its compact form.

`examples/pages/image_creator.rs` is **not modified**. It is what `crates/rux/tests/reference.rs`
measures the library against, and the desktop page staying exactly as task-2008 left it is how this
ticket proves it changed nothing.

---

## 7. `mobile/ios` — the shell

```
mobile/ios/
  README.md                what builds where, and the five things the linker needed
  src/main.rs              the bundle's executable: three lines
  src/lib.rs               the staticlib's one exported symbol, for an existing Xcode project
  RuxMobile/Info.plist     bundle id, orientations, launch screen, Metal requirement
  RuxMobile/main.m         int main() { rux_mobile_main(); }, for that same project
  RuxMobile/rux_mobile.h   void rux_mobile_main(void);
  zig-ios-cc/              the linker cargo calls, and why it is a program rather than a .cmd
  xtool.yml                the route that puts the result on a phone
  build-ios.ps1            compile, link, bundle, sign
```

`build-ios.ps1` does, in order: `rustup target add`, the static library (which needs no SDK and no
linker at all), the executable (which needs both), the bundle, and the signature. It reports the size of
each artefact and, with no SDK, prints the three paths it looked in and what an SDK has to contain.

**There is no `.xcodeproj` in this folder, and that is deliberate.** An iOS application does not need an
Objective-C `main` — winit's iOS backend calls `UIApplicationMain` itself from inside
`eframe::run_native` — so the bundle's executable is a Rust program and the bundle is four files this
script writes. A hand authored `project.pbxproj` would have been a second way to build the same thing,
written on a machine with no Xcode to check it against. The static library and `main.m` are there for the
case that actually needs Xcode: a project that already exists.

**What a Mac still has to do, stated plainly:** install it on a phone and run it. Everything before that
happens here.

---

## 8. `mobile/android` — the shell

```
mobile/android/
  build-android.ps1                cargo-ndk -> .so -> gradle -> adb install -> launch
  settings.gradle.kts, build.gradle.kts, gradle.properties
  app/build.gradle.kts
  app/src/main/AndroidManifest.xml     NativeActivity, android.app.lib_name=main, landscape+portrait
  app/src/main/res/values/themes.xml   NoTitleBar.Fullscreen, so the window is the app's
```

`NativeActivity` rather than `GameActivity`: it is built into Android and needs no Java or Kotlin and
no Maven dependency, and `GameActivity` is only required for AccessKit — which `eframe` `compile_error!`s
in the combination we would otherwise use. The cost is that the Android build has no screen reader
support, and that is written down in the README beside the reason.

`build-android.ps1` builds `aarch64-linux-android` through `cargo-ndk`, drops `libmain.so` into
`app/src/main/jniLibs/arm64-v8a/`, runs Gradle with Android Studio's JBR, and then — if an emulator or
device is attached — installs and launches it and pulls a screenshot with `adb exec-out screencap`.
It reports the APK's path and size either way.

---

## 9. `mobile/preview` — how a phone screen is checked on this machine

A desktop program that runs `mobile/app` inside a phone frame at exactly the logical size of a real
device, with that device's safe area insets forced and `Pointer::Touch` forced:

| | Points | Insets, top/bottom |
|---|---|---|
| iPhone 16 | 393 × 852 | 59 / 34 |
| iPhone 16 Pro Max | 440 × 956 | 62 / 34 |
| iPhone SE | 375 × 667 | 20 / 0 |
| Pixel 8 | 412 × 870 | 24 / 24 |

`--device`, `--screen`, `--theme` and `--capture` are command line switches, for the same reason the
storybook has them: a capture script must reach any state without synthetic input, because activating a
window on Windows switches the virtual desktop with it.

This is what makes the mobile work checkable on the machine it is written on. It is not a substitute
for the Android emulator — it is the thing that shows an iPhone's Dynamic Island inset, which the
emulator cannot.

---

## 10. Testing

The existing tests must pass **unchanged**. That is the evidence that `Screen::desktop` is the old
behaviour rather than a near miss.

New tests, all with no window:

- `Screen`'s breakpoints at 599/600/839/840, and `min_target` being 0 on a mouse;
- `safe()` taking the insets off a rect, and doing nothing with none;
- every component's `measure()` reaching at least 44 points on a touch screen, as a loop over the size
  enums rather than a list somebody has to remember to extend;
- every component's `measure()` being **byte for byte what it is today** on `Screen::desktop`;
- `TopNav` fitting inside 390, 320 and 1200 points, and dropping the label only when it must;
- `Visual::of` never answering `Hover` on a touch pointer;
- the sheet geometry: full width less the margin, bottom anchored, top corners only, bottom inset
  included.

And the things only running it can show, which are §11.

---

## 11. What "done" looks like

- `cargo test` green, the 59 plus the new ones.
- `pwsh mobile/ios/build-ios.ps1` producing a **signed `RuxMobile.app`**, its Mach-O reported as
  `platform=2 (iOS)` with an `LC_CODE_SIGNATURE` in it.
- The Android APK built, installed on the emulator, **launched**, and photographed.
- `mobile/preview` run at 393×852 and at 1200 points wide, both themes, and photographed — the 390pt
  check this ticket asks for, and the full width check beside it.
- Every capture under `_agent_output/task-2032-mobile/`.

---

## 12. Deliberately not in this version

- **A virtual keyboard.** egui shows no software keyboard on either platform; winit 0.30 has no API for
  one, and iOS needs a `UIResponder` and Android a `InputMethodManager` call through JNI. The text
  areas are therefore readable and focusable on a phone and cannot be typed into. This is the largest
  known gap and it is a ticket of its own.
- **Platform gestures**: swipe back, long press, pinch. egui reports multi-touch, so pinch is reachable;
  swipe back is a navigation model rather than a component.
- **Dynamic type.** iOS's accessibility text sizes would scale the whole type scale, which the reference
  tests pin to fixed point sizes. It wants a decision about which of the two wins.
- **A tablet layout.** `SizeClass::Medium` and `Expanded` are defined and reported, and past the compact
  breakpoint the application keeps a readable 720 point column and centres it rather than stretching —
  which is what a one column interface does with room it cannot use. A layout that genuinely uses a
  tablet's width, with the canvas beside the controls, is a different design and is not in this version.
- **Signing and store submission.** §2.3 is the route; it needs your Apple ID and a device.

# `mobile/android` — the Android shell

```powershell
pwsh mobile/android/build-android.ps1                         # build, install, launch
pwsh mobile/android/build-android.ps1 -Capture out.png        # and photograph the screen
pwsh mobile/android/build-android.ps1 -NoInstall              # just the APK
pwsh mobile/android/build-android.ps1 -Release                # a release APK (unsigned)
```

Three steps, each one reporting what it produced: `cargo ndk` cross compiles `mobile/app` for
`arm64-v8a` and `x86_64` and drops `librux_mobile.so` into `app/src/main/jniLibs/<abi>/`; Gradle packages
it; `adb` installs and starts it. Afterwards it prints the application's own log, which is the part that
catches a crash on the first frame — a crashed application leaves a screen that still looks like it is
running.

It needs the Android SDK with an NDK, and a JDK 17 or newer. Android Studio's bundled JBR is used when
`JAVA_HOME` is not set.

## `GameActivity`, and the few lines of Kotlin it needs (task-2189)

The shell used to be the framework's `NativeActivity`, which needs no Java or Kotlin at all. It could not
have a soft keyboard, a screen reader (eframe refuses `accesskit` with `android-native-activity`) or the
real window insets, and all three need a `View`. So the shell is now a `GameActivity` from
`androidx.games:games-activity:4.4.0` (the version `android-activity` 0.6 is written against), and the
Kotlin is kept small and reusable:

| Piece | What it does |
|---|---|
| `rux-shell/RuxActivity.kt` | Draws edge to edge, records the system bar, cutout and keyboard insets, records the keyboard's editor text and action key, and routes the back gesture to Rust while Rust asks for it. An application subclasses it. |
| `rux-shell/RuxBridge.kt` | The values Rust reads once a frame through JNI, and `nativeWake`, which Rust registers so Kotlin can ask for a frame. |
| `shell/` (`rux-android`) | The Rust side: `install`, `Platform::screen`, `Platform::raw_input`, `set_handles_back`, `native_options`, and `with_activity` / `load_class` for an application's own Kotlin. |
| `app/MainActivity.kt` | The example's subclass: one line. |

An application in another repository includes `rux-shell` by path from its `settings.gradle.kts` and
depends on `rux-android` by path from its `Cargo.toml`.

Three things found by running it on the emulator:

- **winit shows GameActivity's keyboard but drops its text.** `set_ime_allowed` calls `show_soft_input`,
  and every `InputEvent::TextEvent` and editor action is then ignored. `RuxActivity.stateChanged` records
  the editor text and `rux-android` diffs it into egui text and backspace events once a frame.
- **`GameActivity_getTextInputState` cannot be called before anything is typed.** It gives
  `android-activity` 0.6.1 a null text pointer, which it turns into a slice: a panic at
  `game_activity/mod.rs:717` in a debug build. That is why the text is read from Kotlin.
- **The emulator with no window runs Vulkan on SwiftShader**, which allows a 16 KiB uniform buffer
  binding. eframe asked for 64 KiB and the device was refused before the first frame.
  `rux_android::native_options` asks for the downlevel limits instead.

## Two ABIs, because the emulator is x86_64

`arm64-v8a` is every phone worth shipping to. `x86_64` is here because the emulator on this machine
reports `sdk_gphone64_x86_64`, and an arm64-only APK installs on it and then runs through binary
translation or not at all.

## The safe area, and why `targetSdk` is now 36

`egui-winit`'s safe area reader is `#[cfg(target_os = "ios")]`, so on Android `InputState::safe_area_insets`
is zero. With `NativeActivity` the only reading was the content rect, and once Android enforces edge to
edge (`targetSdk` 35 and up) that rect is the whole window, which is why this build used to stay on 34.
`RuxActivity.onApplyWindowInsets` now reports the system bars, the display cutout and the keyboard, so
the window is edge to edge on `targetSdk` 36 and `rux` insets the content by what Android reports.

## Two crashes worth knowing about, both found by running it

**The emulator's Vulkan driver segfaults on `vkSetDebugUtilsObjectNameEXT`.** wgpu asks for
`InstanceFlags::DEBUG` by itself in any build with `debug_assertions` on, which makes it name every
object it creates, and `vulkan.ranchu.so` dies inside `vk_common_SetDebugUtilsObjectNameEXT` the first
time it is called. The application had already loaded, enumerated both adapters and chosen Vulkan on the
host's card. `run_android` clears those flags.

**A turned mark killed the process in any debug build**, which was a defect in `rux` itself that no
release build could show — see `crates/rux/tests/draw.rs`, which exists because of it.

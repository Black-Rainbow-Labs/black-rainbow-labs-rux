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

## `NativeActivity`, and what it costs

`NativeActivity` is the framework's own activity for an application whose whole body is a native
library. It needs no Java, no Kotlin and no Maven dependency, which is why there is not a single line of
either in this project and why `compileDebugJavaWithJavac` reports `NO-SOURCE`.

The cost is the screen reader. `eframe` `compile_error!`s `accesskit` together with
`android-native-activity`:

```
error: `accesskit` feature is only available with `android-game-activity`
```

`GameActivity` would allow it and needs a Gradle build that fetches `androidx.games:games-activity` from
Maven and a Java or Kotlin subclass. **So this build has no AccessKit**, while the iOS and desktop builds
do. Every control still calls `Response::widget_info` with its name, so the names are there the moment
the base class changes.

## Two ABIs, because the emulator is x86_64

`arm64-v8a` is every phone worth shipping to. `x86_64` is here because the emulator on this machine
reports `sdk_gphone64_x86_64`, and an arm64-only APK installs on it and then runs through binary
translation or not at all.

## The safe area, and why `targetSdk` is 34

The application was written to draw behind the system bars and keep its own content clear of them, the
way it does on iOS. **That does not work on Android**, and finding out why took a measurement.

`egui-winit`'s safe area reader is `#[cfg(target_os = "ios")]` — it calls `UIWindow.safeAreaInsets`
through `objc2-ui-kit` and there is no Android arm of that file at all. So `InputState::safe_area_insets`
is zero on a phone, and the only reading a `NativeActivity` has is `ANativeActivity`'s content rect,
which `android-activity` keeps as `AndroidApp::content_rect`. Once Android enforces edge to edge, at
`targetSdk` 35 and up, **that rect is the whole window**:

```
content rect Rect { left: 0, top: 0, right: 1080, bottom: 2400 } in a 1080x2400 px window
```

with the nav strip drawn underneath the status bar as a result. The real insets are behind
`View.getRootWindowInsets`, a JNI call that has to run on the Android UI thread while `android_main` is
on its own thread — a ticket of its own.

So `targetSdk` is **34** and the window stays inside the system bars, at which point the same content
rect reads `top: 63, bottom: 2337`, which at a scale of 2.625 is 24 points top and bottom, and `rux`
insets the content by it. Correct rather than approximate.

## Two crashes worth knowing about, both found by running it

**The emulator's Vulkan driver segfaults on `vkSetDebugUtilsObjectNameEXT`.** wgpu asks for
`InstanceFlags::DEBUG` by itself in any build with `debug_assertions` on, which makes it name every
object it creates, and `vulkan.ranchu.so` dies inside `vk_common_SetDebugUtilsObjectNameEXT` the first
time it is called. The application had already loaded, enumerated both adapters and chosen Vulkan on the
host's card. `run_android` clears those flags.

**A turned mark killed the process in any debug build**, which was a defect in `rux` itself that no
release build could show — see `crates/rux/tests/draw.rs`, which exists because of it.

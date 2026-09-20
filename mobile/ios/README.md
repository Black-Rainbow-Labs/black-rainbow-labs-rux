# `mobile/ios` — the iOS shell

```powershell
pwsh mobile/ios/build-ios.ps1                 # compile, link, bundle, sign
pwsh mobile/ios/build-ios.ps1 -StaticOnly     # just librux_mobile.a, for an existing Xcode project
pwsh mobile/ios/build-ios.ps1 -P12 key.p12    # sign with a real Apple certificate instead of ad hoc
```

**All four of those steps happen on Windows.** What comes out is
`target/aarch64-apple-ios/release/RuxMobile.app`: an arm64 Mach-O with
`LC_BUILD_VERSION platform=2 (iOS)`, linking UIKit, Metal, QuartzCore, CoreGraphics, Foundation and
`libobjc`, with an `LC_CODE_SIGNATURE` and a `_CodeSignature/CodeResources` beside it.

## The four steps, and what each one needs

| | Needs | On this machine |
|---|---|---|
| **compile** `aarch64-apple-ios` | nothing Apple at all — a `staticlib` is an archive and nothing in it is resolved | ✅ |
| **link** | an iPhoneOS SDK's `.tbd` stubs, and a linker that emits Mach-O | ✅ `J:/mac-sdk/iPhoneOS.sdk` and zig |
| **bundle** | a directory and an `Info.plist` | ✅ |
| **sign** | Apple's code signing format | ✅ `rcodesign` |
| **install and run** | a device, and Apple's tooling or `xtool` under WSL | ❌ |

The last row is the honest edge of this. There is no iOS Simulator for Windows, so the way to see this
running is a phone.

## The SDK

`build-ios.ps1` looks in `$env:IOS_SDKROOT`, then `J:\mac-sdk\iPhoneOS.sdk`, then `mobile/ios/sdk`, and
if it finds none it says which three it tried and what an SDK has to contain. The one on this machine is
**iPhoneOS 16.5**, from [`theos/sdks`](https://github.com/theos/sdks), sparse-cloned so only that one
SDK came down:

```bash
git clone --filter=blob:none --no-checkout --depth 1 https://github.com/theos/sdks.git J:/mac-sdk/theos-sdks
git -C J:/mac-sdk/theos-sdks sparse-checkout init --cone
git -C J:/mac-sdk/theos-sdks sparse-checkout set iPhoneOS16.5.sdk
git -C J:/mac-sdk/theos-sdks checkout master
# and a junction, so it sits beside MacOSX.sdk under the name the script looks for
New-Item -ItemType Junction -Path J:\mac-sdk\iPhoneOS.sdk -Target J:\mac-sdk\theos-sdks\iPhoneOS16.5.sdk
```

It is Apple's SDK redistributed by somebody else, which is the same licence question `MacOSX.sdk` raised
on task-1995 and which Jason settled the same way. The SDK is not in this repository.

## The linker, which is where the work was

`rustc` drives a C compiler as its linker and there is not one on Windows that emits Mach-O, so
`zig-ios-cc` stands between them. Its source says the whole story; the short version is four things,
each of which stopped the build dead until it was found:

1. **rustc passes a response file, not arguments.** A linker command line this long does not fit on
   Windows, so rustc writes `@<path>` and the file holds several hundred arguments. A filter that reads
   `argv` sees two entries and filters nothing.
2. **`-target arm64-apple-ios10.0.0`** — rustc names the architecture the way Apple does and zig names
   it the way LLVM does, so zig answers `unknown architecture: 'arm64'` and prints its whole list.
3. **`-exported_symbols_list`** — `unsupported linker arg`. It hides a library's private symbols, which
   matters for a `cdylib` and not at all for an executable.
4. **`-isysroot` alone is not enough** — zig answers `unable to find libSystem system library` without
   an explicit `-L` into the SDK's `usr/lib`. That one flag is what stood between this box and an iOS
   binary.

And one more that only shows up at the end: **zig's linker does not reserve header room for the code
signature**, the way Apple's does, so `rcodesign` stopped with `insufficient room to write code
signature load command`. `-Wl,-headerpad_max_install_names` fixes it — a linker flag's problem wearing
a code signing error's clothes.

## Two ways in

**The executable**, which is what `build-ios.ps1` produces. An iOS application does not need an
Objective-C `main`: winit's iOS backend calls `UIApplicationMain` itself from inside
`eframe::run_native`, so `src/main.rs` is three lines and the bundle's executable is a Rust program.

**The static library**, for a project that already exists in Xcode. `-StaticOnly` builds
`librux_mobile.a`; add it and `RuxMobile/rux_mobile.h` to the target and call `rux_mobile_main()` from
the project's own `main`. `RuxMobile/main.m` is that file.

## Getting it onto a phone

Nothing here does this, and the two routes need something only Jason can supply.

**`xtool`** ([xtool.sh](https://github.com/xtool-org/xtool)) builds, signs, installs and launches iOS
applications from WSL with no Xcode and no Mac. It needs a Swift 6.3 toolchain in WSL, `usbmuxd` with
USB passthrough, an iPhone on the cable, an Apple ID, and **the Darwin SDK extracted from an Xcode 26
download**. `xtool.yml` in this folder is the configuration it reads.

**A Mac.** Open the project, or drop `librux_mobile.a` into an existing one, press Run.

Either way the Rust is already built and the bundle is already signed; what is missing is a device.

## What a phone will and will not do

The application draws, scrolls, and responds to touch. **It has no software keyboard** — egui shows none
and winit 0.30 has no API for one, so the text areas are readable and focusable and cannot be typed
into. The safe area insets *are* read on iOS, by `egui-winit` through `objc2-ui-kit`, so the Dynamic
Island and the home indicator are handled properly — which is the one thing iOS does better here than
Android.

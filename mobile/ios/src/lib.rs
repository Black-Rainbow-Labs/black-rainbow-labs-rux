//! The static library an Xcode project links.
//!
//! One exported symbol. An existing Xcode project drags `librux_mobile.a` in, declares
//! `void rux_mobile_main(void);` — `RuxMobile/rux_mobile.h` is that declaration — and calls it from its
//! own `main`, which is what `RuxMobile/main.m` does.
//!
//! **Producing this needs no Apple SDK and no linker**, because a `staticlib` is an archive of object
//! files and nothing is resolved. That is why the whole application compiles for `aarch64-apple-ios` on
//! a Windows machine, and why the only step that needed Apple's SDK was the link.

/// Start the application. Never returns: `UIApplicationMain` does not come back.
///
/// `extern "C"` with no result, because an `int` nobody can ever read would be a lie about the control
/// flow.
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn rux_mobile_main() {
    rux_mobile::run_ios();
}

/// Off iOS this archive has nothing in it, and it still has to say what it is.
#[cfg(not(target_os = "ios"))]
#[no_mangle]
pub extern "C" fn rux_mobile_ios_is_only_for_ios() {}

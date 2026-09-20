//! The Android shell: one symbol, and the crate type that carries it.
//!
//! `NativeActivity` loads `librux_mobile.so` and calls `android_main` in it. Everything that follows is
//! `rux-mobile`, which has no platform code in it at all.
//!
//! **This is a crate rather than a `crate-type` on `mobile/app` because cargo builds every crate type in
//! a library's list.** With `cdylib` on the application crate, an iOS build of `mobile/ios` also tried to
//! link `librux_mobile.dylib` — and a dylib is a link, and a link wants every Apple framework resolved,
//! on a build that only needed an rlib.

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    rux_mobile::run_android(app);
}

/// Off Android there is nothing to export, and an empty `cdylib` is fine — but a crate that silently
/// compiles to nothing is a crate nobody notices has stopped working, so this says what it is.
#[cfg(not(target_os = "android"))]
#[no_mangle]
pub extern "C" fn rux_mobile_android_is_only_for_android() {}

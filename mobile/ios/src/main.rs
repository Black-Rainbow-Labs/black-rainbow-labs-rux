//! The iOS bundle's executable.
//!
//! **An iOS application does not need an Objective-C `main`, and this one does not have one.** winit's
//! iOS backend calls `UIApplicationMain` itself from inside `eframe::run_native`, so the whole of the
//! Apple side of this program is the one call below. The other route — an existing Xcode project that
//! links `librux_mobile.a` and calls `rux_mobile_main()` from its own `main.m` — is `src/lib.rs`.
//!
//! Off iOS it builds and refuses rather than not building at all: every member of this workspace has to
//! compile on the machine the work is done on, and a crate that only exists for one target is a crate
//! nobody notices has stopped compiling.

#[cfg(target_os = "ios")]
fn main() {
    rux_mobile::run_ios();
}

#[cfg(not(target_os = "ios"))]
fn main() {
    eprintln!(
        "RuxMobile is the iOS bundle's executable and only does anything on iOS.\n\
         Build it with: pwsh mobile/ios/build-ios.ps1"
    );
    std::process::exit(2);
}

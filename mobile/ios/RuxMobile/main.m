//
//  The Objective-C entry point, for an Xcode project that links `librux_mobile.a`.
//
//  It is not used by `build-ios.ps1`, which builds the Rust `bin` instead — winit's iOS backend calls
//  `UIApplicationMain` itself, so a Rust `main` is a complete iOS application on its own. This file is
//  here for the other direction: dropping the static library into a project that already exists.
//
#import "rux_mobile.h"

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;
    rux_mobile_main();
    return 0;
}

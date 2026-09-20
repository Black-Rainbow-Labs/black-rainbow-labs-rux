/*
 * The one symbol `librux_mobile.a` exports.
 *
 * For the route where an existing Xcode project links the static library rather than using the
 * executable this repository's own script builds: add `librux_mobile.a` to the target, add this header,
 * and call `rux_mobile_main()` from the project's `main`. It does not return — winit calls
 * `UIApplicationMain` inside it.
 */
#ifndef RUX_MOBILE_H
#define RUX_MOBILE_H

void rux_mobile_main(void);

#endif /* RUX_MOBILE_H */

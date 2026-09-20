//! The linker cargo calls for `aarch64-apple-ios`.
//!
//! `rustc` drives a C compiler as its linker, and there is not one on Windows that emits Mach-O — so
//! this hands the job to zig, with the flags that make Apple's frameworks resolvable and without the
//! arguments zig's linker refuses.
//!
//! # Why this is a program rather than a `.cmd`
//!
//! It was a `.cmd` first, and two things killed that:
//!
//! 1. **`cmd` does not pass a linker command line through unharmed.** rustc's is several hundred
//!    arguments long and holds commas, braces and equals signs; `%*` came out the other side mangled and
//!    the build failed with `'m' is not recognized as an internal or external command`, five times, from
//!    the tail of `-lc -lm`.
//! 2. **Some of rustc's arguments have to be dropped rather than passed on**, and a batch file cannot
//!    filter an argument list.
//!
//! # The response file, which is why the first filter did nothing
//!
//! On Windows a linker command line this long does not fit, so **rustc writes the whole thing to a file
//! and passes `@<path>`**. The first version of this program filtered its own `argv` and found nothing
//! to filter, because `argv` was two entries long: the response file and nothing else. So the file is
//! expanded here, filtered, and written out again for zig — which is also how the result stays inside
//! Windows' own limit.
//!
//! rustc's format for that file is one argument a line, with `\` and `"` backslash escaped, in UTF-8
//! unless the linker is `link.exe`.
//!
//! # What is dropped, and why
//!
//! - **`-exported_symbols_list <file>`** — zig's Mach-O linker answers `unsupported linker arg:
//!   -exported_symbols_list`. rustc passes it to hide a library's private symbols, which matters for a
//!   `cdylib` and not at all for an executable.
//! - **`-syslibroot <dir>`** — likewise unsupported, and `-isysroot` says the same thing.
//! - **`-target arm64-apple-ios10.0.0`** — rustc names the architecture the way Apple does and zig names
//!   it the way LLVM does, so zig answers `unknown architecture: 'arm64'` and prints its whole list. The
//!   `-target aarch64-ios` added below is the same triple in zig's spelling, and the deployment target
//!   rustc was carrying comes back as `-mios-version-min`.
//!
//! # What is added
//!
//! `-isysroot` alone is not enough: zig answers `unable to find libSystem system library` without an
//! explicit `-L` into the SDK's `usr/lib`. That one flag is what stood between this machine and an iOS
//! binary.
//!
//! `-headerpad_max_install_names` leaves room in the Mach-O header for the code signature load command,
//! which Apple's linker reserves and zig's does not.
//!
//! `RUX_ZIG` and `IOS_SDKROOT` are set by `build-ios.ps1`. `RUX_IOS_CC_LOG`, when set, names a file the
//! expanded argument list is appended to — which is the only way to see what rustc actually passed.

use std::io::Write as _;
use std::process::{exit, Command};

fn main() {
    let zig = std::env::var("RUX_ZIG")
        .unwrap_or_else(|_| fail("RUX_ZIG is not set: it is the path to zig.exe"));
    let sdk = std::env::var("IOS_SDKROOT")
        .unwrap_or_else(|_| fail("IOS_SDKROOT is not set: it is the path to iPhoneOS.sdk"));

    let given: Vec<String> = std::env::args().skip(1).collect();
    let expanded = expand_response_files(&given);
    log(&expanded);

    let mut kept: Vec<String> = vec![
        "cc".to_owned(),
        "-target".to_owned(),
        "aarch64-ios".to_owned(),
        "-isysroot".to_owned(),
        sdk.clone(),
        format!("-L{sdk}/usr/lib"),
        format!("-F{sdk}/System/Library/Frameworks"),
        "-mios-version-min=13.0".to_owned(),
        // Room in the Mach-O header for the code signature load command that `rcodesign` adds later.
        // Apple's own linker reserves it; zig's does not, and without it signing the bundle stops with
        // `insufficient room to write code signature load command` — which is a linker flag's problem
        // wearing a code signing error's clothes.
        "-Wl,-headerpad_max_install_names".to_owned(),
    ];

    let mut args = expanded.into_iter();
    while let Some(arg) = args.next() {
        match refused(&arg) {
            // The flag and the value after it both go.
            Refused::WithValue => {
                let _ = args.next();
            }
            Refused::Alone => {}
            Refused::No => kept.push(arg),
        }
    }

    // Back into a response file of our own: the list is longer than a Windows command line.
    let response = match write_response(&kept[1..]) {
        Ok(path) => path,
        Err(problem) => fail(&format!("could not write the response file: {problem}")),
    };

    let status = Command::new(&zig).arg("cc").arg(format!("@{response}")).status();
    let _ = std::fs::remove_file(&response);
    match status {
        Ok(status) => exit(status.code().unwrap_or(1)),
        Err(problem) => fail(&format!("could not run {zig}: {problem}")),
    }
}

/// Replace every `@<path>` with the arguments inside it.
///
/// One level deep, which is all rustc produces.
fn expand_response_files(args: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        match arg.strip_prefix('@') {
            Some(path) => match std::fs::read(path) {
                Ok(bytes) => out.extend(parse_response(&bytes)),
                Err(problem) => fail(&format!("could not read the response file {path}: {problem}")),
            },
            None => out.push(arg.clone()),
        }
    }
    out
}

/// One argument a line, with `\` and `"` backslash escaped. UTF-16 when it starts with a byte order
/// mark, which is what rustc writes for `link.exe`, and UTF-8 otherwise.
fn parse_response(bytes: &[u8]) -> Vec<String> {
    let text = match bytes {
        [0xFF, 0xFE, rest @ ..] => {
            let units: Vec<u16> =
                rest.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
            String::from_utf16_lossy(&units)
        }
        _ => String::from_utf8_lossy(bytes).into_owned(),
    };

    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut out = String::with_capacity(line.len());
            let mut escaped = false;
            for character in line.chars() {
                match (escaped, character) {
                    (true, _) => {
                        out.push(character);
                        escaped = false;
                    }
                    (false, '\\') => escaped = true,
                    (false, _) => out.push(character),
                }
            }
            out
        })
        .collect()
}

/// Write the arguments back out in the format zig reads them in.
fn write_response(args: &[String]) -> std::io::Result<String> {
    let path = std::env::temp_dir().join(format!("zig-ios-cc-{}.rsp", std::process::id()));
    let mut file = std::fs::File::create(&path)?;
    for arg in args {
        let escaped = arg.replace('\\', "\\\\").replace('"', "\\\"");
        writeln!(file, "\"{escaped}\"")?;
    }
    Ok(path.to_string_lossy().into_owned())
}

enum Refused {
    No,
    Alone,
    WithValue,
}

/// Whether zig's linker refuses this argument, and whether the thing after it goes with it.
///
/// rustc writes each of these two ways — the bare flag followed by its value, and `-Wl,-flag` followed
/// by `-Wl,<value>` — so both spellings are matched.
fn refused(arg: &str) -> Refused {
    const WITH_VALUE: [&str; 3] = ["-exported_symbols_list", "-syslibroot", "-target"];
    for flag in WITH_VALUE {
        if arg == flag || arg == format!("-Wl,{flag}") {
            return Refused::WithValue;
        }
        // `-Wl,-flag,<value>` carries its own value and takes nothing after it.
        if arg.starts_with(&format!("-Wl,{flag},")) {
            return Refused::Alone;
        }
    }
    Refused::No
}

fn log(args: &[String]) {
    let Ok(path) = std::env::var("RUX_IOS_CC_LOG") else { return };
    let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "---- {} args", args.len());
    for arg in args {
        let _ = writeln!(file, "{arg}");
    }
}

fn fail(message: &str) -> ! {
    eprintln!("zig-ios-cc: {message}");
    exit(2)
}

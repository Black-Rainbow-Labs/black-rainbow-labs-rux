# Build the iOS application on Windows: compile, link, bundle and sign, with no Mac anywhere.
#
#   pwsh mobile/ios/build-ios.ps1                 build RuxMobile.app and sign it ad hoc
#   pwsh mobile/ios/build-ios.ps1 -Dev            the debug profile
#   pwsh mobile/ios/build-ios.ps1 -StaticOnly     just the .a, for an existing Xcode project
#   pwsh mobile/ios/build-ios.ps1 -P12 key.p12    sign with a real Apple certificate instead
#
# ## What this machine can and cannot do, and which step is which
#
#   compile   ✅  no Apple SDK involved at all — `staticlib` is an archive and nothing is resolved
#   link      ✅  zig emits Mach-O; it needs iPhoneOS.sdk's stubs, which `-SdkRoot` finds
#   bundle    ✅  an `.app` is a directory with an Info.plist in it
#   sign      ✅  `rcodesign` is Apple's code signing format implemented in Rust
#   install   ❌  needs a device, and Apple's own tooling or `xtool` under WSL
#   run       ❌  there is no iOS Simulator for Windows
#
# So what comes out of here is a signed `.app` that has never been near a Mac. Getting it onto a phone
# is the one step that still needs something else — see README.md.

[CmdletBinding()]
param(
    # `-Debug` is one of PowerShell's own common parameters once `[CmdletBinding()]` is on, and
    # redeclaring it fails at call time rather than at parse time. Hence `-Dev`.
    [switch] $Dev,
    [switch] $StaticOnly,
    [string] $SdkRoot = '',
    [string] $Zig = '',
    [string] $P12 = '',
    [string] $P12Password = '',
    [string] $Out = ''
)

$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$root = Split-Path -Parent (Split-Path -Parent $here)
# `$profile` is one of PowerShell's own automatic variables, so the build profile is not called that.
$profileName = if ($Dev) { 'debug' } else { 'release' }
$target = 'aarch64-apple-ios'

# ---------------------------------------------------------------- the SDK, named rather than hoped for

if (-not $SdkRoot) { $SdkRoot = $env:IOS_SDKROOT }
$looked = @()
foreach ($candidate in @($SdkRoot, 'J:\mac-sdk\iPhoneOS.sdk', (Join-Path $here 'sdk\iPhoneOS.sdk'))) {
    if (-not $candidate) { continue }
    $looked += $candidate
    if (Test-Path (Join-Path $candidate 'System\Library\Frameworks\UIKit.framework\UIKit.tbd')) {
        $SdkRoot = $candidate
        break
    }
    $SdkRoot = ''
}

if (-not $SdkRoot -and -not $StaticOnly) {
    Write-Warning 'No iPhoneOS SDK, so the link cannot happen. The static library can still be built:'
    Write-Warning '  pwsh mobile/ios/build-ios.ps1 -StaticOnly'
    Write-Warning ''
    Write-Warning 'Looked in:'
    $looked | ForEach-Object { Write-Warning "  $_" }
    Write-Warning ''
    Write-Warning 'An SDK goes in any of those, or is named with -SdkRoot. It must hold'
    Write-Warning 'System/Library/Frameworks/UIKit.framework/UIKit.tbd targeting arm64-ios.'
    throw 'no iPhoneOS SDK'
}

# ---------------------------------------------------------------- zig, and the linker shim

if (-not $Zig) { $Zig = $env:RUX_ZIG }
if (-not $Zig) {
    $Zig = 'C:\jason\dev\unluminous\tools\cross\bin\zig\zig.exe'
}
if (-not $StaticOnly -and -not (Test-Path $Zig)) {
    throw "No zig at $Zig. `pwsh tools/cross/fetch-toolchain.ps1` in the unluminous checkout fetches one, or name it with -Zig."
}

$rustupTargets = & rustup target list --installed
if ($rustupTargets -notcontains $target) {
    Write-Output "rustup target add $target"
    & rustup target add $target
}

# ---------------------------------------------------------------- 1. the static library
#
# No linker and no SDK: an archive is object files with an index, and nothing in it is resolved. This is
# the step that proves the whole application compiles for an iPhone on a Windows machine.

Write-Output "cargo build --target $target -p rux-mobile-ios --lib  ($profileName)"
$libArgs = @('build', '--target', $target, '-p', 'rux-mobile-ios', '--lib')
if (-not $Dev) { $libArgs += '--release' }
$cargo = Start-Process -FilePath 'cargo' -ArgumentList $libArgs -WorkingDirectory $root -NoNewWindow -Wait -PassThru
if ($cargo.ExitCode -ne 0) { throw "cargo build --lib failed with $($cargo.ExitCode)" }

$staticLib = Join-Path $root "target\$target\$profileName\librux_mobile.a"
if (-not (Test-Path $staticLib)) { throw "cargo reported success and wrote no archive. Looked for $staticLib" }
Write-Output ("librux_mobile.a  {0} MB   {1}" -f [math]::Round((Get-Item $staticLib).Length / 1MB, 1), $staticLib)

if ($StaticOnly) {
    Write-Output ''
    Write-Output 'Add that archive and RuxMobile/rux_mobile.h to an Xcode project, and call'
    Write-Output 'rux_mobile_main() from its main. RuxMobile/main.m is that file.'
    return
}

# ---------------------------------------------------------------- 2. the executable
#
# The one step that needs the SDK. `zig-ios-cc` is the linker cargo calls; its own source says what it
# has to change about rustc's command line and why.

Write-Output ''
Write-Output 'cargo build --release -p zig-ios-cc   (the linker shim, for the host)'
$shimBuild = Start-Process -FilePath 'cargo' -ArgumentList @('build', '--release', '-p', 'zig-ios-cc') `
    -WorkingDirectory $root -NoNewWindow -Wait -PassThru
if ($shimBuild.ExitCode -ne 0) { throw "could not build zig-ios-cc: $($shimBuild.ExitCode)" }
$shim = Join-Path $root 'target\release\zig-ios-cc.exe'

$env:RUX_ZIG = $Zig
$env:IOS_SDKROOT = $SdkRoot
$env:CARGO_TARGET_AARCH64_APPLE_IOS_LINKER = $shim

Write-Output ''
Write-Output "SDK   $SdkRoot"
Write-Output "zig   $Zig"
Write-Output "cargo build --target $target -p rux-mobile-ios --bin RuxMobile  ($profileName)"
$binArgs = @('build', '--target', $target, '-p', 'rux-mobile-ios', '--bin', 'RuxMobile')
if (-not $Dev) { $binArgs += '--release' }
$cargo = Start-Process -FilePath 'cargo' -ArgumentList $binArgs -WorkingDirectory $root -NoNewWindow -Wait -PassThru
if ($cargo.ExitCode -ne 0) { throw "cargo build --bin failed with $($cargo.ExitCode)" }

$executable = Join-Path $root "target\$target\$profileName\RuxMobile"
if (-not (Test-Path $executable)) { throw "cargo reported success and wrote no executable. Looked for $executable" }
Write-Output ("RuxMobile  {0} MB" -f [math]::Round((Get-Item $executable).Length / 1MB, 1))

# ---------------------------------------------------------------- 3. the bundle
#
# An `.app` for iOS is flat: the executable and Info.plist at the top, no Contents/MacOS the way a macOS
# bundle has. Everything that is in it is written here, so nothing depends on a tool that only exists on
# a Mac.

if (-not $Out) { $Out = Join-Path $root "target\$target\$profileName\RuxMobile.app" }
if (Test-Path $Out) { Remove-Item -Recurse -Force -Confirm:$false $Out }
New-Item -ItemType Directory -Force -Path $Out | Out-Null
Copy-Item $executable (Join-Path $Out 'RuxMobile')
Copy-Item (Join-Path $here 'RuxMobile\Info.plist') (Join-Path $Out 'Info.plist')

Write-Output ''
Write-Output "bundle  $Out"

# ---------------------------------------------------------------- 4. the signature
#
# Every iOS binary is signed, including one that will only ever be side loaded: the kernel refuses an
# unsigned Mach-O outright. An ad hoc signature is what a build with no certificate gets, and it is
# enough for a jailbroken device or for re-signing later; `-P12` signs with a real identity instead.

$rcodesign = Join-Path (Split-Path -Parent $Zig) '..\rcodesign.exe'
$rcodesign = [System.IO.Path]::GetFullPath($rcodesign)
if (-not (Test-Path $rcodesign)) {
    Write-Warning "No rcodesign at $rcodesign, so the bundle is unsigned. iOS will refuse to run it."
    return
}

$signArgs = @('sign')
if ($P12) {
    $signArgs += @('--p12-file', $P12)
    if ($P12Password) { $signArgs += @('--p12-password', $P12Password) }
}
$signArgs += $Out
$sign = Start-Process -FilePath $rcodesign -ArgumentList $signArgs -NoNewWindow -Wait -PassThru
if ($sign.ExitCode -ne 0) { throw "rcodesign sign failed with $($sign.ExitCode)" }

$kind = if ($P12) { 'with the supplied certificate' } else { 'ad hoc' }
Write-Output "signed $kind"
& $rcodesign print-signature-info (Join-Path $Out 'RuxMobile') |
    Select-String -Pattern 'code_directory_enabled|signature_type|team_name|cdhash|identifier|platform' |
    Select-Object -First 8 | ForEach-Object { Write-Output "  $_" }

Write-Output ''
Write-Output 'Done. What is NOT possible here is installing and running it: that needs a device and'
Write-Output 'either Apple''s own tooling or xtool under WSL. README.md says what each route needs.'

# Build the Android application, install it and launch it.
#
#   pwsh mobile/android/build-android.ps1                  build, install and launch
#   pwsh mobile/android/build-android.ps1 -Release         a release APK (unsigned)
#   pwsh mobile/android/build-android.ps1 -NoInstall       just build
#   pwsh mobile/android/build-android.ps1 -Capture out.png install, launch and photograph the screen
#
# Three steps, and each one says what it produced:
#
#   1. `cargo ndk` cross compiles `mobile/android/glue` — which is `mobile/app` plus the one symbol
#      `android-activity` calls — for each ABI, and drops `librux_mobile.so` into
#      `app/src/main/jniLibs/<abi>/`.
#   2. Gradle packages it with the Kotlin shell (`rux-shell`: `RuxActivity` and `RuxBridge`) and the
#      GameActivity library they extend.
#   3. `adb` installs it on whatever device or emulator is attached and starts it.
#
# The Rust build is a separate step rather than a Gradle task that shells out to cargo, because cargo's
# own output — which is where a cross compilation error appears — is hidden inside a Gradle task, and
# because `mobile/ios/build-ios.ps1` has to do the same cross compilation with no Gradle anywhere.

[CmdletBinding()]
param(
    [switch] $Release,
    [switch] $NoInstall,
    [string] $Capture = '',
    [string[]] $Abis = @('arm64-v8a', 'x86_64'),
    [string] $Serial = '',
    [int] $ApiLevel = 28
)

$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$root = Split-Path -Parent (Split-Path -Parent $here)

# ---------------------------------------------------------------- the toolchain, named rather than hoped for

$sdk = $env:ANDROID_HOME
if (-not $sdk) { $sdk = $env:ANDROID_SDK_ROOT }
if (-not $sdk) { $sdk = Join-Path $env:LOCALAPPDATA 'Android\Sdk' }
if (-not (Test-Path $sdk)) {
    throw "No Android SDK. Looked at ANDROID_HOME, ANDROID_SDK_ROOT and $sdk."
}

# The newest installed NDK, so a later one does not need this file edited.
$ndkRoot = Join-Path $sdk 'ndk'
if (-not (Test-Path $ndkRoot)) { throw "No NDK under $ndkRoot. Install one from the SDK Manager." }
$ndk = Get-ChildItem $ndkRoot -Directory | Sort-Object Name -Descending | Select-Object -First 1
if (-not $ndk) { throw "No NDK under $ndkRoot. Install one from the SDK Manager." }

# Gradle needs a JDK 17 or newer. Android Studio ships one, which is the one this machine has.
if (-not $env:JAVA_HOME) {
    $jbr = 'C:\Program Files\Android\Android Studio\jbr'
    if (Test-Path $jbr) { $env:JAVA_HOME = $jbr }
}
if (-not $env:JAVA_HOME) {
    throw 'No JAVA_HOME, and Android Studio''s bundled JBR is not where it usually is. Gradle needs a JDK 17 or newer.'
}

if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
    throw 'cargo-ndk is not installed. Run: cargo install cargo-ndk'
}

$env:ANDROID_HOME = $sdk
$env:ANDROID_NDK_HOME = $ndk.FullName

$adb = Join-Path $sdk 'platform-tools\adb.exe'
$profile = if ($Release) { 'release' } else { 'debug' }

Write-Output "SDK      $sdk"
Write-Output "NDK      $($ndk.Name)"
Write-Output "JDK      $env:JAVA_HOME"
Write-Output ''

# ---------------------------------------------------------------- 1. the Rust

$jniLibs = Join-Path $here 'app\src\main\jniLibs'
$cargoArgs = @('ndk')
foreach ($abi in $Abis) { $cargoArgs += @('-t', $abi) }
$cargoArgs += @('-P', "$ApiLevel", '-o', 'mobile/android/app/src/main/jniLibs', 'build', '-p', 'rux-mobile-android')
if ($Release) { $cargoArgs += '--release' }

Write-Output "cargo ndk $($Abis -join ', ') -P $ApiLevel ($profile)"
$cargo = Start-Process -FilePath 'cargo' -ArgumentList $cargoArgs -WorkingDirectory $root -NoNewWindow -Wait -PassThru
if ($cargo.ExitCode -ne 0) { throw "cargo ndk failed with $($cargo.ExitCode)" }

foreach ($abi in $Abis) {
    $so = Join-Path $jniLibs "$abi\librux_mobile.so"
    if (-not (Test-Path $so)) { throw "cargo ndk reported success and wrote no library. Looked for $so" }
    $soSize = [math]::Round((Get-Item $so).Length / 1MB, 1)
    Write-Output "librux_mobile.so  $abi  $soSize MB"
}

# ---------------------------------------------------------------- 2. the APK

$task = if ($Release) { 'assembleRelease' } else { 'assembleDebug' }
Write-Output ''
Write-Output "gradlew $task"
$gradle = Start-Process -FilePath (Join-Path $here 'gradlew.bat') -ArgumentList @($task, '--no-daemon') `
    -WorkingDirectory $here -NoNewWindow -Wait -PassThru
if ($gradle.ExitCode -ne 0) { throw "gradle $task failed with $($gradle.ExitCode)" }

$apk = Get-ChildItem (Join-Path $here "app\build\outputs\apk\$profile") -Filter '*.apk' |
    Select-Object -First 1
if (-not $apk) { throw "Gradle reported success and wrote no APK." }
$apkSize = [math]::Round($apk.Length / 1MB, 1)
Write-Output ''
Write-Output "APK  $($apk.FullName)  $apkSize MB"

if ($NoInstall) { return }

# ---------------------------------------------------------------- 3. install, launch, and look

$devices = @(& $adb devices | Select-Object -Skip 1 | Where-Object { $_ -match '\sdevice$' } |
    ForEach-Object { ($_ -split '\s+')[0] })
if (-not $devices) {
    Write-Warning 'No device or emulator attached, so nothing was installed.'
    Write-Warning "Start one with: $sdk\emulator\emulator.exe -avd <name>   (-list-avds to see them)"
    return
}
# **Every adb call names its device.** Without `-s`, adb answers "more than one device/emulator" the
# moment a second transport appears — and one appears on its own during a streamed install, so a script
# that left it out installed correctly and then failed to launch what it had just installed.
if (-not $Serial) { $Serial = $devices[0] }
$adbArgs = @('-s', $Serial)

Write-Output ''
Write-Output "installing on $Serial"
& $adb @adbArgs install -r $apk.FullName | Write-Output

$package = 'com.blackrainbowlabs.rux'
& $adb @adbArgs shell am start -n "$package/.MainActivity" | Write-Output

if ($Capture) {
    # The first frame needs the surface, the swapchain and the font atlas, so it is worth waiting for.
    Start-Sleep -Seconds 6
    $dir = Split-Path -Parent $Capture
    if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
    # **Through a file on the device, not down a pipe.** `adb exec-out screencap -p` writes PNG bytes to
    # standard output, and PowerShell's pipeline carries objects rather than bytes — it decodes them as
    # text on the way through, and `Set-Content -AsByteStream` then refuses what it is handed. Writing
    # the file on the device and pulling it keeps the bytes bytes.
    $onDevice = '/sdcard/rux-capture.png'
    & $adb @adbArgs shell screencap -p $onDevice | Out-Null
    & $adb @adbArgs pull $onDevice $Capture | Out-Null
    & $adb @adbArgs shell rm -f $onDevice | Out-Null
    if (Test-Path $Capture) {
        Write-Output "screen -> $Capture"
    } else {
        Write-Warning "screencap produced nothing at $Capture"
    }

    # And the application's own log, because a crash on the first frame leaves a running-looking screen.
    $log = & $adb @adbArgs logcat -d -t 400 | Where-Object { $_ -match 'rux|RustStdout|GameActivity|eframe|wgpu' }
    if ($log) {
        Write-Output ''
        Write-Output 'the last thing it said:'
        $log | Select-Object -Last 15 | Write-Output
    }
}

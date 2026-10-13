# Photograph every mobile screen, on every device, in both themes, without touching the keyboard or the
# pointer.
#
# The same rules as `capture-stories.ps1`: `--background` opens the window without taking the foreground
# and `--device`, `--screen` and `--theme` reach any state from the command line, so the pass needs no
# synthetic input at all. Activating a window on Windows switches the virtual desktop with it, which is
# not something a capture run may do to somebody who is working.
#
#   pwsh tools/capture-mobile.ps1
#   pwsh tools/capture-mobile.ps1 -Devices iphone-16 -Themes dark-neumorphic
#   pwsh tools/capture-mobile.ps1 -Wide            # the expanded size class as well

param(
    [string[]]$Devices = @(),
    [string[]]$Screens = @('create', 'gallery', 'controls'),
    [string[]]$Themes = @('dark-neumorphic', 'light-neumorphic'),
    [switch]$Wide,
    [string]$Out = '_agent_output/task-2032-mobile/screens',
    [int]$SettleMs = 2600
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
# A build with CARGO_TARGET_DIR set puts the program there instead of under target/.
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
$exe = Join-Path $targetDir 'release/rux-mobile-preview.exe'
if (-not (Test-Path $exe)) { throw "Build it first: cargo build --release. Looked for $exe" }

if ($Devices.Count -eq 0) {
    # `--list` prints the devices under a `devices:` heading and the screens under a `screens:` one, so
    # the first column of the indented lines up to `screens:` is the device list.
    $inDevices = $false
    $Devices = & $exe --list | ForEach-Object {
        if ($_ -match '^devices:') { $inDevices = $true; return }
        if ($_ -match '^screens:') { $inDevices = $false; return }
        if ($inDevices -and $_ -match '^\s+(\S+)') { $matches[1] }
    } | Where-Object { $_ }
}

$outDir = Join-Path $root $Out
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function Capture {
    param([string[]]$Arguments, [string]$File, [string]$What)
    $process = Start-Process -FilePath $exe -ArgumentList $Arguments -PassThru -WorkingDirectory $root
    try {
        Start-Sleep -Milliseconds $SettleMs
        if ($process.HasExited) {
            Write-Warning "$What : the window exited with $($process.ExitCode)"
            return
        }
        # One bad frame must not cost the rest of the pass.
        try {
            & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $process.Id -Out $File | Out-Null
            Write-Output "$What -> $File"
        } catch {
            Write-Warning "$What : $($_.Exception.Message)"
        }
    } finally {
        if (-not $process.HasExited) {
            Stop-Process -Id $process.Id -Force -Confirm:$false -ErrorAction SilentlyContinue
        }
    }
}

foreach ($theme in $Themes) {
    foreach ($device in $Devices) {
        foreach ($screen in $Screens) {
            $file = Join-Path $outDir "$theme--$device--$screen.png"
            Capture -Arguments @('--background', '--theme', $theme, '--device', $device, '--screen', $screen) `
                -File $file -What "$theme / $device / $screen"
        }
    }
    if ($Wide) {
        foreach ($screen in $Screens) {
            $file = Join-Path $outDir "$theme--wide-1200--$screen.png"
            Capture -Arguments @('--background', '--theme', $theme, '--wide', '--screen', $screen) `
                -File $file -What "$theme / wide 1200 / $screen"
        }
    }
}

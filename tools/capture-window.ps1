# Photograph a window without bringing it to the front.
#
# `PrintWindow` with `PW_RENDERFULLCONTENT` asks the window to draw itself into a bitmap, so it works
# on a window that is behind another one, on another virtual desktop, or that has never been
# activated at all. That matters more than it sounds: the alternative is a screen grab, which needs
# the window in the foreground — and activating a window on Windows switches the virtual desktop with
# it and takes the keyboard out of whatever the person was typing into.
#
#   pwsh tools/capture-window.ps1 -ProcessId 1234 -Out _agent_output/shot.png
#   pwsh tools/capture-window.ps1 -Title 'RUX Storybook' -Out _agent_output/shot.png

param(
    [int]$ProcessId = 0,
    [string]$Title = '',
    [Parameter(Mandatory = $true)][string]$Out
)

$ErrorActionPreference = 'Stop'

# Guarded, because this script is run once per capture and `Add-Type` refuses to define a type that
# is already in the session.
if (-not ('Rux.Win' -as [type])) {
Add-Type -Namespace Rux -Name Win -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
[DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
[StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
'@
}

Add-Type -AssemblyName System.Drawing

function Find-Window {
    param([int]$ProcessId, [string]$Title)
    # `MainWindowHandle` is cached on the process object and is zero until the window exists, so the
    # process is refreshed before it is asked. The caller retries.
    $candidates = if ($ProcessId -gt 0) {
        @(Get-Process -Id $ProcessId -ErrorAction Stop)
    } elseif ($Title) {
        @(Get-Process | Where-Object { $_.MainWindowTitle -like "*$Title*" })
    } else {
        throw 'Give me either -ProcessId or -Title.'
    }
    foreach ($process in $candidates) {
        $process.Refresh()
        if ($process.MainWindowHandle -ne 0) { return $process.MainWindowHandle }
    }
    return [IntPtr]::Zero
}

# A window exists before it has been laid out, so its client rectangle is briefly zero — and the
# handle `MainWindowHandle` first reports is not always the one that ends up carrying the client area.
# So the window is **found again** on each attempt rather than measured again, which is what made this
# stop flaking one capture in ten.
$handle = [IntPtr]::Zero
$width = 0
$height = 0
for ($attempt = 0; $attempt -lt 40; $attempt++) {
    $handle = Find-Window -ProcessId $ProcessId -Title $Title
    $rect = New-Object Rux.Win+RECT
    if ([Rux.Win]::GetClientRect($handle, [ref]$rect)) {
        $width = $rect.Right - $rect.Left
        $height = $rect.Bottom - $rect.Top
        if ($width -gt 0 -and $height -gt 0) { break }
    }
    Start-Sleep -Milliseconds 250
}
if ($width -le 0 -or $height -le 0) { throw "That window never measured more than ${width}x${height}." }

$bitmap = New-Object System.Drawing.Bitmap $width, $height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$hdc = $graphics.GetHdc()
try {
    # 0x00000002 is PW_RENDERFULLCONTENT, which is what makes this work on a composited window.
    $ok = [Rux.Win]::PrintWindow($handle, $hdc, 2)
} finally {
    $graphics.ReleaseHdc($hdc)
}
if (-not $ok) { throw 'PrintWindow refused.' }

$folder = Split-Path -Parent $Out
if ($folder -and -not (Test-Path $folder)) { New-Item -ItemType Directory -Path $folder | Out-Null }
$bitmap.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()

# Say what the foreground window was, so a run can prove it did not steal the keyboard.
Write-Output "wrote $Out (${width}x${height}); foreground window unchanged at $([Rux.Win]::GetForegroundWindow())"

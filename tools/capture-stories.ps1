# Photograph every story, in both themes, without touching the keyboard or the pointer.
#
# `--background` opens the window without taking the foreground and `--story` opens it straight onto
# a story, so the whole pass needs no synthetic input at all. That matters on Windows: activating a
# window switches the virtual desktop with it, which is not something a capture run may do to
# somebody who is working.
#
#   pwsh tools/capture-stories.ps1
#   pwsh tools/capture-stories.ps1 -Stories Button,Select -Themes dark-neumorphic
#   pwsh tools/capture-stories.ps1 -Size 1440x2400 -Out docs/images/stories   # whole stories, for docs/

param(
    [string[]]$Stories = @(),
    [string[]]$Themes = @('dark-neumorphic', 'light-neumorphic'),
    [string]$Out = '_agent_output/task-2008-rux/stories',
    [int]$SettleMs = 2600,
    # The window's inner size in points, such as 1440x2400. A story is taller than the default
    # 1440x920, so the pictures in docs/ are taken taller to show a whole story.
    [string]$Size = '',
    # Pass `--open` as well, which opens the dropdowns and the dialog.
    [switch]$Open,
    # Added to each file name after the story, such as `open`, so a second pass does not overwrite the first.
    [string]$Suffix = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
# A build with CARGO_TARGET_DIR set puts the program there instead of under target/.
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
$exe = Join-Path $targetDir 'release/rux-storybook.exe'
if (-not (Test-Path $exe)) { throw "Build it first: cargo build --release. Looked for $exe" }

if ($Stories.Count -eq 0) {
    # `--list` prints "<section> <name> <rule>" a line; the name is the second column, which can hold
    # a space, so the rule is split off at the two spaces before it.
    $Stories = & $exe --list | ForEach-Object {
        if ($_ -match '^\S+\s+(.+?)\s{2,}') { $matches[1].Trim() }
    } | Where-Object { $_ }
}

$outDir = Join-Path $root $Out
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

foreach ($theme in $Themes) {
    foreach ($story in $Stories) {
        $slug = ($story -replace '[^A-Za-z0-9]+', '-').ToLower().Trim('-')
        if ($Suffix) { $slug = "$slug--$Suffix" }
        $file = Join-Path $outDir "$theme--$slug.png"
        # A story name has a space in it — `Icon Button`, `Top Nav` — and `Start-Process` splits its
        # argument list on spaces unless each one is quoted, so the program was seeing `Icon` and then
        # a switch it did not know and exiting with 2. Quoted here rather than renamed, because the
        # names on the screen are the names a person reads.
        $arguments = @('--background', '--theme', "`"$theme`"", '--story', "`"$story`"")
        if ($Size) { $arguments += @('--size', $Size) }
        if ($Open) { $arguments += '--open' }
        $process = Start-Process -FilePath $exe -ArgumentList $arguments -PassThru -WorkingDirectory $root
        try {
            Start-Sleep -Milliseconds $SettleMs
            if ($process.HasExited) {
                Write-Warning "$theme / $story : the window exited with $($process.ExitCode)"
                continue
            }
            # A window that will not photograph is reported and the pass carries on: one bad frame
            # must not cost the other forty-five.
            try {
                & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $process.Id -Out $file | Out-Null
                Write-Output "$theme / $story -> $file"
            } catch {
                Write-Warning "$theme / $story : $($_.Exception.Message)"
            }
        } finally {
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -Confirm:$false -ErrorAction SilentlyContinue
            }
        }
    }
}

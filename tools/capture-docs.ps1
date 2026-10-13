# Take the pictures in docs/images/: every story in both themes, each one whole, cropped to the story.
#
# A story can be taller than the screen, and Windows will not open a window taller than the screen. So
# each story is photographed in slices, one `--scroll` each, and the slices are joined. Each picture is
# then cut to the story's own page: the storybook's top bar, its sidebar and the empty ground under the
# last state are cut away, because the pages in docs/ show the component and not the storybook.
#
# The same rules as capture-stories.ps1 hold: `--background` and `--story` reach every state from the
# command line, so the pass sends no keystroke and no click and never takes the foreground.
#
#   pwsh tools/capture-docs.ps1
#   pwsh tools/capture-docs.ps1 -Stories Button,'Chat Parts' -Themes dark-neumorphic

param(
    [string[]]$Stories = @(),
    [string[]]$Themes = @('dark-neumorphic', 'light-neumorphic'),
    [string]$Out = 'docs/images',
    [int]$SettleMs = 2600
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = Split-Path -Parent $PSScriptRoot
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
$exe = Join-Path $targetDir 'release/rux-storybook.exe'
if (-not (Test-Path $exe)) { throw "Build it first: cargo build --release -p rux-storybook. Looked for $exe" }

# The storybook's own layout, in points, from storybook/src/app.rs. The storybook draws one pixel to a
# point at 100% display scaling, which is what these pictures are taken at.
$TopBar = 64       # TOPBAR
$StoryHead = 66    # the story's title and rule, above the page that scrolls
$Sidebar = 264     # SIDEBAR
$Scrollbar = 12    # the scrollbar's lane at the right edge of the page
$WindowWidth = 1440
$WindowHeight = 2000
$MaxSlices = 4
# Ground left under the last state, so a picture does not end at the edge of a shadow.
$BottomMargin = 36

# Stories whose open states are worth their own picture, taken again with `--open`.
$OpenStories = @('Select', 'Project Menu', 'Modal')

if ($Stories.Count -eq 0) {
    $Stories = & $exe --list | ForEach-Object {
        if ($_ -match '^\S+\s+(.+?)\s{2,}') { $matches[1].Trim() }
    } | Where-Object { $_ }
}
$fullBleed = @(& $exe --list | Where-Object { $_ -match '^Pages\s' } | ForEach-Object {
    if ($_ -match '^\S+\s+(.+?)\s{2,}') { $matches[1].Trim() }
})

$outDir = Join-Path $root $Out
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$scratch = Join-Path ([IO.Path]::GetTempPath()) "rux-capture-docs-$PID"
New-Item -ItemType Directory -Force -Path $scratch | Out-Null

# Open the storybook with these switches, photograph it, and close it. Answers the file, or $null.
function Take {
    param([string[]]$Arguments, [string]$File)
    $process = Start-Process -FilePath $exe -ArgumentList $Arguments -PassThru -WorkingDirectory $root
    try {
        Start-Sleep -Milliseconds $SettleMs
        if ($process.HasExited) { Write-Warning "the window exited with $($process.ExitCode): $Arguments"; return $null }
        & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $process.Id -Out $File | Out-Null
        return $File
    } finally {
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force -Confirm:$false -ErrorAction SilentlyContinue }
    }
}

# Whether a row of a bitmap is all ground: every sampled pixel within a few levels of $Ground.
function Test-GroundRow {
    param([System.Drawing.Bitmap]$Bitmap, [int]$Y, [System.Drawing.Color]$Ground)
    for ($x = 0; $x -lt $Bitmap.Width; $x += 6) {
        $pixel = $Bitmap.GetPixel($x, $Y)
        if ([Math]::Abs($pixel.R - $Ground.R) + [Math]::Abs($pixel.G - $Ground.G) + [Math]::Abs($pixel.B - $Ground.B) -gt 9) {
            return $false
        }
    }
    return $true
}

# The last row of a bitmap that is not ground, or -1 when every row is.
function Find-LastContentRow {
    param([System.Drawing.Bitmap]$Bitmap, [System.Drawing.Color]$Ground)
    for ($y = $Bitmap.Height - 1; $y -ge 0; $y -= 2) {
        if (-not (Test-GroundRow -Bitmap $Bitmap -Y $y -Ground $Ground)) { return [Math]::Min($Bitmap.Height - 1, $y + 1) }
    }
    return -1
}

# Copy a rectangle of one bitmap into a new one.
function Copy-Region {
    param([System.Drawing.Bitmap]$Source, [System.Drawing.Rectangle]$Rect)
    return $Source.Clone($Rect, $Source.PixelFormat)
}

# Photograph one story whole: slices down the page, cut to the page, joined, and trimmed.
function Capture-Story {
    param([string]$Theme, [string]$Story, [string]$File, [switch]$Open)
    $base = @('--background', '--theme', "`"$Theme`"", '--story', "`"$Story`"", '--size', "${WindowWidth}x${WindowHeight}")
    if ($Open) { $base += '--open' }
    $bodyTop = $TopBar + $StoryHead
    $sliceHeight = $WindowHeight - $bodyTop
    $pieces = New-Object System.Collections.Generic.List[System.Drawing.Bitmap]
    $ground = $null
    try {
        for ($index = 0; $index -lt $MaxSlices; $index++) {
            $shot = Join-Path $scratch "slice-$index.png"
            $taken = Take -Arguments ($base + @('--scroll', [string]($index * $sliceHeight))) -File $shot
            if (-not $taken) { break }
            $bitmap = [System.Drawing.Bitmap]::FromFile($shot)
            try {
                $width = $bitmap.Width - $Sidebar - $Scrollbar
                if ($index -eq 0) {
                    # The ground of the page, read where no story draws: the bottom right corner.
                    $ground = $bitmap.GetPixel($bitmap.Width - $Scrollbar - 4, $bitmap.Height - 4)
                    $rect = New-Object System.Drawing.Rectangle $Sidebar, $TopBar, $width, ($bitmap.Height - $TopBar)
                } else {
                    $rect = New-Object System.Drawing.Rectangle $Sidebar, $bodyTop, $width, ($bitmap.Height - $bodyTop)
                }
                $piece = Copy-Region -Source $bitmap -Rect $rect
            } finally { $bitmap.Dispose() }
            $last = Find-LastContentRow -Bitmap $piece -Ground $ground
            $pieces.Add($piece)
            # A slice that ends in ground is the end of the story.
            if ($last -lt $piece.Height - 40) { break }
        }
        if ($pieces.Count -eq 0) { return }
        $total = 0
        foreach ($piece in $pieces) { $total += $piece.Height }
        $joined = New-Object System.Drawing.Bitmap $pieces[0].Width, $total
        $graphics = [System.Drawing.Graphics]::FromImage($joined)
        $y = 0
        foreach ($piece in $pieces) { $graphics.DrawImageUnscaled($piece, 0, $y); $y += $piece.Height }
        $graphics.Dispose()
        $last = Find-LastContentRow -Bitmap $joined -Ground $ground
        $height = [Math]::Min($joined.Height, [Math]::Max(200, $last + $BottomMargin))
        $final = Copy-Region -Source $joined -Rect (New-Object System.Drawing.Rectangle 0, 0, $joined.Width, $height)
        $final.Save($File, [System.Drawing.Imaging.ImageFormat]::Png)
        $final.Dispose()
        $joined.Dispose()
        Write-Output "$Theme / $Story -> $File ($($pieces.Count) slices, $height px)"
    } finally {
        foreach ($piece in $pieces) { $piece.Dispose() }
    }
}

try {
    foreach ($theme in $Themes) {
        foreach ($story in $Stories) {
            $slug = ($story -replace '[^A-Za-z0-9]+', '-').ToLower().Trim('-')
            $folder = Join-Path $outDir $slug
            New-Item -ItemType Directory -Force -Path $folder | Out-Null
            $file = Join-Path $folder "$theme.png"
            if ($fullBleed -contains $story) {
                # A page fills the window, so it is photographed as the window at its default size.
                $taken = Take -Arguments @('--background', '--theme', "`"$theme`"", '--story', "`"$story`"") -File $file
                if ($taken) { Write-Output "$theme / $story -> $file (the whole window)" }
                continue
            }
            Capture-Story -Theme $theme -Story $story -File $file
            if ($OpenStories -contains $story) {
                if ($story -eq 'Modal') {
                    # A dialog covers the whole window, so its open state is the whole window.
                    $taken = Take -Arguments @('--background', '--theme', "`"$theme`"", '--story', "`"$story`"", '--open') -File (Join-Path $folder "$theme--open.png")
                    if ($taken) { Write-Output "$theme / $story open -> the whole window" }
                } else {
                    Capture-Story -Theme $theme -Story $story -File (Join-Path $folder "$theme--open.png") -Open
                }
            }
        }
        # The storybook itself, sidebar and all, for the readme.
        $file = Join-Path $outDir "storybook--$theme.png"
        $taken = Take -Arguments @('--background', '--theme', "`"$theme`"", '--story', '"Chat Parts"') -File $file
        if ($taken) { Write-Output "$theme / the storybook -> $file" }
    }
} finally {
    Remove-Item -LiteralPath (Join-Path $scratch 'slice-0.png'), (Join-Path $scratch 'slice-1.png'), (Join-Path $scratch 'slice-2.png'), (Join-Path $scratch 'slice-3.png') -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $scratch -Force -ErrorAction SilentlyContinue
}

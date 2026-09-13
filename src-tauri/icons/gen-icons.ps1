Add-Type -AssemblyName System.Drawing

$outDir = "D:\Projects\SGameStudio\src-tauri\icons"
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

function New-IconBitmap([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.Clear([System.Drawing.Color]::FromArgb(255, 13, 17, 23))

    $scale = $size / 256.0
    $pad = 18 * $scale

    # Rounded-square background
    $bg = New-Object System.Drawing.Drawing2D.GraphicsPath
    $r = (28 * $scale)
    $x0 = $pad; $y0 = $pad
    $x1 = $size - $pad; $y1 = $size - $pad
    $bg.AddArc($x0, $y0, 2*$r, 2*$r, 180, 90)
    $bg.AddArc($x1 - 2*$r, $y0, 2*$r, 2*$r, 270, 90)
    $bg.AddArc($x1 - 2*$r, $y1 - 2*$r, 2*$r, 2*$r, 0, 90)
    $bg.AddArc($x0, $y1 - 2*$r, 2*$r, 2*$r, 90, 90)
    $bg.CloseFigure()
    $bgBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 22, 27, 34))
    $g.FillPath($bgBrush, $bg)

    # Violet accent ring / emblem
    $violet = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 124, 58, 237), (10 * $scale))
    $violet.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $violet.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

    $cx = $size / 2.0
    $cy = $size / 2.0

    # Cube-ish diamond emblem
    $top    = New-Object System.Drawing.PointF($cx, ($cy - 55 * $scale))
    $right  = New-Object System.Drawing.PointF(($cx + 55 * $scale), $cy)
    $bottom = New-Object System.Drawing.PointF($cx, ($cy + 55 * $scale))
    $left   = New-Object System.Drawing.PointF(($cx - 55 * $scale), $cy)

    $emblem = New-Object System.Drawing.SolidBrush($violet.Color)
    $diamond = [System.Drawing.PointF[]]@($top, $right, $bottom, $left)
    $g.FillPolygon($emblem, $diamond)

    # Inner audio-ish accent
    $accent = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 250, 250, 250))
    $inner = [System.Drawing.PointF[]]@(
        (New-Object System.Drawing.PointF(($cx - 16 * $scale), ($cy - 16 * $scale))),
        (New-Object System.Drawing.PointF(($cx + 16 * $scale), ($cy - 16 * $scale))),
        (New-Object System.Drawing.PointF(($cx + 16 * $scale), ($cy + 16 * $scale))),
        (New-Object System.Drawing.PointF(($cx - 16 * $scale), ($cy + 16 * $scale)))
    )
    $g.FillPolygon($accent, $inner)

    $g.Dispose()
    return $bmp
}

# Save PNG sizes
foreach ($s in @(32, 128, 256)) {
    $b = New-IconBitmap $s
    $b.Save((Join-Path $outDir "icon_$($s)x$($s).png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $b.Dispose()
}
# Primary png
$b = New-IconBitmap 256
$pngPath = Join-Path $outDir "icon.png"
$b.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

# Build ICO embedding the 256 PNG (Vista+ supports PNG-compressed ICO entries)
$pngBytes = [System.IO.File]::ReadAllBytes($pngPath)
$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)
# ICONDIR
$bw.Write([uint16]0)              # reserved
$bw.Write([uint16]1)              # type: icon
$bw.Write([uint16]1)              # count
# ICONDIRENTRY
$bw.Write([byte]0)                # width (0 => 256)
$bw.Write([byte]0)                # height
$bw.Write([byte]0)                # color count
$bw.Write([byte]0)                # reserved
$bw.Write([uint16]1)              # planes
$bw.Write([uint16]32)             # bit count
$bw.Write([uint32]$pngBytes.Length)  # bytes in resource
$bw.Write([uint32]22)             # image offset (6 + 16)
$bw.Write($pngBytes)
$bw.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $outDir "icon.ico"), $ms.ToArray())
$bw.Dispose()
$b.Dispose()

Write-Host "Generated icons in $outDir"
Get-ChildItem $outDir | Select-Object Name, Length

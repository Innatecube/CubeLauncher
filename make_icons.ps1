Add-Type -AssemblyName System.Drawing

$srcPath = "apps\app\icons\icon (1).png"
$src = [System.Drawing.Image]::FromFile((Resolve-Path $srcPath))

function Save-ResizedPNG($size, $outPath) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $g.DrawImage($src, 0, 0, $size, $size)
    $bmp.Save((Resolve-Path ".").Path + "\" + $outPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
    Write-Host "Saved $outPath ($size x $size)"
}

# Save all PNG sizes
Save-ResizedPNG 32  "apps\app\icons\32x32.png"
Save-ResizedPNG 128 "apps\app\icons\128x128.png"
Save-ResizedPNG 256 "apps\app\icons\128x128@2x.png"
Save-ResizedPNG 512 "apps\app\icons\icon.png"

# Also update the Square tiles used by Windows
Save-ResizedPNG 30  "apps\app\icons\Square30x30Logo.png"
Save-ResizedPNG 44  "apps\app\icons\Square44x44Logo.png"
Save-ResizedPNG 71  "apps\app\icons\Square71x71Logo.png"
Save-ResizedPNG 89  "apps\app\icons\Square89x89Logo.png"
Save-ResizedPNG 107 "apps\app\icons\Square107x107Logo.png"
Save-ResizedPNG 142 "apps\app\icons\Square142x142Logo.png"
Save-ResizedPNG 150 "apps\app\icons\Square150x150Logo.png"
Save-ResizedPNG 284 "apps\app\icons\Square284x284Logo.png"
Save-ResizedPNG 310 "apps\app\icons\Square310x310Logo.png"
Save-ResizedPNG 50  "apps\app\icons\StoreLogo.png"

# Build multi-size ICO
$sizes = @(16, 24, 32, 48, 64, 128, 256)
$imgData = @()

foreach ($s in $sizes) {
    $bmp = New-Object System.Drawing.Bitmap($s, $s)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $g.DrawImage($src, 0, 0, $s, $s)
    $pngMs = New-Object System.IO.MemoryStream
    $bmp.Save($pngMs, [System.Drawing.Imaging.ImageFormat]::Png)
    $imgData += , $pngMs.ToArray()
    $g.Dispose()
    $bmp.Dispose()
    $pngMs.Dispose()
}

$ms = New-Object System.IO.MemoryStream

# ICO header: reserved=0, type=1 (ICO), count
$count = $sizes.Count
[byte[]]$header = 0, 0, 1, 0, ($count -band 0xFF), (($count -shr 8) -band 0xFF)
$ms.Write($header, 0, 6)

# Directory starts after header (6 bytes) + directory entries (16 bytes each)
$dataOffset = 6 + 16 * $count
$offsets = @()
$cur = $dataOffset
foreach ($d in $imgData) {
    $offsets += $cur
    $cur += $d.Length
}

for ($i = 0; $i -lt $count; $i++) {
    $s = $sizes[$i]
    $d = $imgData[$i]
    $w = if ($s -eq 256) { 0 } else { $s }
    $h = if ($s -eq 256) { 0 } else { $s }
    $len = $d.Length
    $off = $offsets[$i]
    [byte[]]$entry = @(
        [byte]$w, [byte]$h,
        0, 0,  # color count, reserved
        1, 0,  # planes
        32, 0, # bits per pixel
        [byte]($len -band 0xFF), [byte](($len -shr 8) -band 0xFF), [byte](($len -shr 16) -band 0xFF), [byte](($len -shr 24) -band 0xFF),
        [byte]($off -band 0xFF), [byte](($off -shr 8) -band 0xFF), [byte](($off -shr 16) -band 0xFF), [byte](($off -shr 24) -band 0xFF)
    )
    $ms.Write($entry, 0, 16)
}

foreach ($d in $imgData) {
    $ms.Write($d, 0, $d.Length)
}

$icoBytes = $ms.ToArray()
$ms.Dispose()

[System.IO.File]::WriteAllBytes((Resolve-Path ".").Path + "\apps\app\icons\icon.ico", $icoBytes)
[System.IO.File]::WriteAllBytes((Resolve-Path ".").Path + "\apps\app\icons\favicon.ico", $icoBytes)
Write-Host "icon.ico written ($($icoBytes.Length) bytes, $count sizes)"

$src.Dispose()
Write-Host "All icons generated successfully!"

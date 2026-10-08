Add-Type -AssemblyName System.Drawing

$srcPath = Join-Path $PSScriptRoot "..\upload.png"
if (-not (Test-Path $srcPath)) {
    Write-Error "Source icon not found at $srcPath"
    exit 1
}

$srcImg = [System.Drawing.Image]::FromFile($srcPath)

function Resize-Image($img, $width, $height, $outPath) {
    $bmp = New-Object System.Drawing.Bitmap($width, $height)
    $graph = [System.Drawing.Graphics]::FromImage($bmp)
    $graph.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $graph.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graph.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graph.DrawImage($img, 0, 0, $width, $height)
    $graph.Dispose()
    $bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "Created: $outPath ($($width)x$($height))"
}

$iconsDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
$publicDir = Join-Path $PSScriptRoot "..\public"

Resize-Image $srcImg 32 32 (Join-Path $iconsDir "32x32.png")
Resize-Image $srcImg 128 128 (Join-Path $iconsDir "128x128.png")
Resize-Image $srcImg 256 256 (Join-Path $iconsDir "128x128@2x.png")
Resize-Image $srcImg 256 256 (Join-Path $publicDir "icon.png")

$srcImg.Dispose()

# Create .ico using png data
$pngBytes = [System.IO.File]::ReadAllBytes((Join-Path $iconsDir "128x128.png"))
$icoStream = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($icoStream)

# ICONDIR header
$bw.Write([UInt16]0) # Reserved
$bw.Write([UInt16]1) # Type 1 = ICO
$bw.Write([UInt16]1) # 1 image

# ICONDIRENTRY
$bw.Write([Byte]128) # Width (128)
$bw.Write([Byte]128) # Height (128)
$bw.Write([Byte]0)   # Colors (0 = 256+)
$bw.Write([Byte]0)   # Reserved
$bw.Write([UInt16]1) # Color planes
$bw.Write([UInt16]32) # Bits per pixel
$bw.Write([UInt32]$pngBytes.Length) # Image size
$bw.Write([UInt32]22) # Image offset (6 + 16 = 22)

$bw.Write($pngBytes)
$bw.Flush()

[System.IO.File]::WriteAllBytes((Join-Path $iconsDir "icon.ico"), $icoStream.ToArray())
[System.IO.File]::WriteAllBytes((Join-Path $publicDir "favicon.ico"), $icoStream.ToArray())
$icoStream.Dispose()

Write-Host "All icons generated successfully!"

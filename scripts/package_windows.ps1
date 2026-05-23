$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistDir = Join-Path $RootDir "dist\windows\RustSurvivors"

Set-Location $RootDir

cargo build -p game --bin survivor --release

if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Force $DistDir | Out-Null

Copy-Item (Join-Path $RootDir "target\release\survivor.exe") $DistDir
Copy-Item -Recurse (Join-Path $RootDir "assets") $DistDir
Copy-Item (Join-Path $RootDir "docs\ASSET_LICENSES.md") $DistDir
Copy-Item (Join-Path $RootDir "docs\audio_assets.md") $DistDir

Get-ChildItem $DistDir -Recurse -Force |
    Where-Object { $_.Name -like "._*" -or $_.Name -eq ".DS_Store" } |
    Remove-Item -Force

if (-not (Test-Path (Join-Path $DistDir "survivor.exe"))) {
    throw "survivor.exe was not copied"
}
if (-not (Test-Path (Join-Path $DistDir "assets"))) {
    throw "assets directory was not copied"
}
if (-not (Test-Path (Join-Path $DistDir "ASSET_LICENSES.md"))) {
    throw "ASSET_LICENSES.md was not copied"
}
if (-not (Test-Path (Join-Path $DistDir "audio_assets.md"))) {
    throw "audio_assets.md was not copied"
}

$AppleDoubleFiles = Get-ChildItem $DistDir -Recurse -Force |
    Where-Object { $_.Name -like "._*" -or $_.Name -eq ".DS_Store" }
if ($AppleDoubleFiles) {
    throw "macOS metadata files remained in package"
}

$ManifestPath = Join-Path $DistDir "PACKAGE_MANIFEST.sha256"
$ManifestLines = Get-ChildItem $DistDir -Recurse -File -Force |
    Where-Object { $_.FullName -ne $ManifestPath } |
    Sort-Object FullName |
    ForEach-Object {
        $relativePath = [System.IO.Path]::GetRelativePath($DistDir, $_.FullName) -replace "\\", "/"
        $hash = (Get-FileHash -Algorithm SHA256 $_.FullName).Hash.ToLowerInvariant()
        "$hash  ./$relativePath"
    }
$ManifestLines | Set-Content -Encoding ascii $ManifestPath
if (-not (Test-Path $ManifestPath)) {
    throw "PACKAGE_MANIFEST.sha256 was not created"
}

& (Join-Path $RootDir "scripts\verify_windows_package.ps1") $DistDir

Write-Host "Packaged Windows folder: $DistDir"

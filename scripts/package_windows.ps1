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

if (-not (Test-Path (Join-Path $DistDir "survivor.exe"))) {
    throw "survivor.exe was not copied"
}
if (-not (Test-Path (Join-Path $DistDir "assets"))) {
    throw "assets directory was not copied"
}

Write-Host "Packaged Windows folder: $DistDir"

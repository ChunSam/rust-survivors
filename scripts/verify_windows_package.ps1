# Verifies a packaged Windows build: required files present, manifest hashes intact,
# and the zip carries the assets the executable needs at runtime.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/verify_windows_package.ps1

param(
    [string]$DistDir,
    [string]$ZipPath
)

$ErrorActionPreference = 'Stop'

$RootDir = Split-Path -Parent $PSScriptRoot
if (-not $DistDir) { $DistDir = Join-Path $RootDir 'dist/windows/RustSurvivors' }
if (-not $ZipPath) { $ZipPath = Join-Path $RootDir 'dist/windows/RustSurvivors-windows-x64.zip' }

function Assert-Path {
    param([string]$Path, [string]$What)
    if (-not (Test-Path -LiteralPath $Path)) { throw "$What missing: $Path" }
}

Assert-Path (Join-Path $DistDir 'survivor.exe') 'executable'
Assert-Path (Join-Path $DistDir 'assets/textures/survivor') 'texture assets'
Assert-Path (Join-Path $DistDir 'assets/audio') 'audio assets'
Assert-Path (Join-Path $DistDir 'assets/data') 'data assets'
Assert-Path (Join-Path $DistDir 'ASSET_LICENSES.md') 'license table'
Assert-Path (Join-Path $DistDir 'audio_assets.md') 'audio asset notes'

$manifestPath = Join-Path $DistDir 'PACKAGE_MANIFEST.sha256'
Assert-Path $manifestPath 'package manifest'

foreach ($line in Get-Content -LiteralPath $manifestPath) {
    if (-not $line.Trim()) { continue }
    $expected, $relative = $line -split '\s+\./', 2
    $file = Join-Path $DistDir $relative
    Assert-Path $file "manifest entry"
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash.ToLower()
    if ($actual -ne $expected) { throw "hash mismatch for ${relative}: expected $expected, got $actual" }
}

Assert-Path $ZipPath 'zip archive'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $ZipPath))
try {
    $entries = $zip.Entries | ForEach-Object { $_.FullName }
} finally {
    $zip.Dispose()
}

# The bare .exe is the failure mode this package exists to prevent - assert the zip
# actually carries assets alongside it. Entries must use forward slashes to extract
# correctly outside Windows.
$required = @(
    'RustSurvivors/survivor.exe',
    'RustSurvivors/assets/textures/survivor/survivor_atlas.png',
    'RustSurvivors/assets/data/weapons.ron'
)
foreach ($entry in $required) {
    if ($entries -notcontains $entry) { throw "zip is missing $entry" }
}
if (-not ($entries | Where-Object { $_ -like 'RustSurvivors/assets/audio/*.mp3' })) {
    throw 'zip carries no audio tracks'
}
if ($entries | Where-Object { $_ -like '*\*' }) { throw 'zip entries contain backslash separators' }

Write-Output "Verified Windows package:"
Write-Output "  $DistDir"
Write-Output "  $ZipPath ($($entries.Count) entries)"

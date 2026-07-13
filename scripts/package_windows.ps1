# Builds the release binary and packages it with its assets into a shareable zip.
#
# The game loads textures/audio through `assets/...` relative paths, so the executable
# is useless on its own: shipping survivor.exe alone leaves every texture on the engine's
# magenta fallback. Always hand out the zip produced here, never the bare .exe.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1

$ErrorActionPreference = 'Stop'

$RootDir = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $RootDir 'dist/windows'
$DistDir = Join-Path $DistRoot 'RustSurvivors'
$ZipPath = Join-Path $DistRoot 'RustSurvivors-windows-x64.zip'

Set-Location $RootDir

cargo build -p game --bin survivor --release --locked
if ($LASTEXITCODE -ne 0) { throw "release build failed (exit $LASTEXITCODE)" }

if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

Copy-Item (Join-Path $RootDir 'target/release/survivor.exe') $DistDir
Copy-Item (Join-Path $RootDir 'assets') $DistDir -Recurse
Copy-Item (Join-Path $RootDir 'docs/ASSET_LICENSES.md') $DistDir
Copy-Item (Join-Path $RootDir 'docs/audio_assets.md') $DistDir

# Same manifest format as the macOS package: `<sha256>  ./<path>`, byte-ordered by path
# (LC_ALL=C sort there, ordinal comparison here) so the two stay diffable.
$manifestPath = Join-Path $DistDir 'PACKAGE_MANIFEST.sha256'
$hashes = @{}
foreach ($file in Get-ChildItem -Path $DistDir -Recurse -File) {
    $relative = $file.FullName.Substring($DistDir.Length + 1).Replace('\', '/')
    $hashes[$relative] = (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash.ToLower()
}
$relatives = [string[]]$hashes.Keys
[Array]::Sort($relatives, [StringComparer]::Ordinal)
$lines = $relatives | ForEach-Object { "$($hashes[$_])  ./$_" }
Set-Content -LiteralPath $manifestPath -Value $lines -Encoding utf8

# Entries are written by hand rather than with Compress-Archive or ZipFile::CreateFromDirectory:
# both name entries with the platform separator on Windows PowerShell 5.1, and a zip full of
# `assets\...` entries extracts to literal backslash filenames on macOS/Linux. The zip spec wants
# forward slashes. Everything is nested under `RustSurvivors/` so unzipping yields one folder.
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipStream = [System.IO.File]::Open($ZipPath, [System.IO.FileMode]::Create)
$archive = New-Object System.IO.Compression.ZipArchive($zipStream, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($file in Get-ChildItem -Path $DistDir -Recurse -File) {
        $relative = $file.FullName.Substring($DistDir.Length + 1).Replace('\', '/')
        $entry = $archive.CreateEntry("RustSurvivors/$relative", [System.IO.Compression.CompressionLevel]::Optimal)
        $entryStream = $entry.Open()
        $fileStream = [System.IO.File]::OpenRead($file.FullName)
        try { $fileStream.CopyTo($entryStream) } finally { $fileStream.Dispose(); $entryStream.Dispose() }
    }
} finally {
    $archive.Dispose()
    $zipStream.Dispose()
}

& (Join-Path $PSScriptRoot 'verify_windows_package.ps1') -DistDir $DistDir -ZipPath $ZipPath
if ($LASTEXITCODE -ne 0) { throw "package verification failed (exit $LASTEXITCODE)" }

$zipMb = [math]::Round((Get-Item $ZipPath).Length / 1MB, 1)
Write-Output "Packaged Windows folder: $DistDir"
Write-Output "Packaged Windows zip: $ZipPath ($zipMb MB)"

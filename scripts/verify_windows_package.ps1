$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistDir = if ($args.Count -gt 0) { $args[0] } else { Join-Path $RootDir "dist\windows\RustSurvivors" }

$Required = @(
    "survivor.exe",
    "assets",
    "ASSET_LICENSES.md",
    "audio_assets.md",
    "PACKAGE_MANIFEST.sha256"
)

foreach ($Item in $Required) {
    $Path = Join-Path $DistDir $Item
    if (-not (Test-Path $Path)) {
        throw "Missing package item: $Path"
    }
}

$MetadataFiles = Get-ChildItem $DistDir -Recurse -Force |
    Where-Object { $_.Name -like "._*" -or $_.Name -eq ".DS_Store" }
if ($MetadataFiles) {
    throw "macOS metadata files remained in Windows package"
}

Push-Location $DistDir
try {
    Get-Content PACKAGE_MANIFEST.sha256 | ForEach-Object {
        if ($_ -notmatch "^([a-f0-9]{64})  \./(.+)$") {
            throw "Invalid manifest line: $_"
        }
        $Expected = $Matches[1]
        $Relative = $Matches[2] -replace "/", [IO.Path]::DirectorySeparatorChar
        if (-not (Test-Path $Relative)) {
            throw "Manifest path missing: $Relative"
        }
        $Actual = (Get-FileHash -Algorithm SHA256 $Relative).Hash.ToLowerInvariant()
        if ($Actual -ne $Expected) {
            throw "Hash mismatch: $Relative"
        }
    }
} finally {
    Pop-Location
}

Write-Host "Verified Windows package: $DistDir"

# AgnesStudio build and package script
$ErrorActionPreference = "Stop"

$cargoToml = Get-Content "Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') { $version = $matches[1] } else { Write-Error "Cannot parse version from Cargo.toml"; exit 1 }
Write-Host "Version: $version" -ForegroundColor Cyan

Write-Host "Building release..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Error "cargo build --release failed"; exit 1 }

$exePath = "target\release\agnes-studio.exe"
if (-not (Test-Path $exePath)) { Write-Error "Build output not found: $exePath"; exit 1 }
Write-Host "Build output: $exePath ($((Get-Item $exePath).Length) bytes)" -ForegroundColor Green

$iscc = $null
$candidates = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)
# Set ISCC_HOME env var if Inno Setup is installed elsewhere
if ($env:ISCC_HOME) {
    $candidates = @($env:ISCC_HOME + "\\ISCC.exe") + $candidates
}
foreach ($c in $candidates) { if (Test-Path $c) { $iscc = $c; break } }
if (-not $iscc) { $found = Get-Command ISCC.exe -ErrorAction SilentlyContinue; if ($found) { $iscc = $found.Source } }
if (-not $iscc) {
    Write-Warning "ISCC.exe not found. Install Inno Setup 6 from:"
    Write-Warning "  https://jrsoftware.org/isinfo.php"
    Write-Warning ""
    Write-Warning "If installed elsewhere, set ISCC_HOME or run manually:"
    Write-Warning "  & 'YOUR_PATH\\ISCC.exe' /DMyAppVersion='$version' setup.iss"
    exit 1
}
Write-Host "Inno Setup: $iscc" -ForegroundColor Green

if (-not (Test-Path "dist")) { New-Item -ItemType Directory -Path "dist" | Out-Null }

Write-Host "Compiling Inno Setup installer..." -ForegroundColor Yellow
& $iscc /DMyAppVersion="$version" setup.iss
if ($LASTEXITCODE -ne 0) { Write-Error "ISCC compilation failed"; exit 1 }

$setupExe = "dist\AgnesStudio-Setup-$version.exe"
if (Test-Path $setupExe) {
    $size = (Get-Item $setupExe).Length
    Write-Host "Done: $setupExe ($size bytes)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Release steps:" -ForegroundColor Cyan
    Write-Host "  1. Create a GitHub Release with tag v$version"
    Write-Host "  2. Upload $setupExe to the Release assets"
    Write-Host "  3. Release title: AgnesStudio v$version"
} else { Write-Error "Output not found: $setupExe"; exit 1 }

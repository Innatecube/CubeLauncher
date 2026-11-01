# Build CubeLauncher Inno Setup Installer
$ErrorActionPreference = "Stop"

$isccCandidates = @(
    "C:\Users\$env:USERNAME\AppData\Local\Programs\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$iscc = $null
foreach ($cand in $isccCandidates) {
    if (Test-Path $cand) {
        $iscc = $cand
        break
    }
}

if (-not $iscc) {
    Write-Error "Inno Setup compiler (ISCC.exe) was not found. Please install Inno Setup 6."
    exit 1
}

Write-Host "Using Inno Setup compiler at: $iscc" -ForegroundColor Cyan
Write-Host "Compiling package\CubeLauncher.iss..." -ForegroundColor Cyan

$tempDir = Join-Path $env:TEMP "CubeLauncherDist"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

& $iscc /O"$tempDir" "package\CubeLauncher.iss"

if ($LASTEXITCODE -eq 0) {
    if (Test-Path "$tempDir\CubeLauncher-Setup.exe") {
        Move-Item "$tempDir\CubeLauncher-Setup.exe" "dist\CubeLauncher-Setup.exe" -Force
    }
    Write-Host "`nSuccessfully created dist\CubeLauncher-Setup.exe!" -ForegroundColor Green
} else {
    Write-Error "Installer compilation failed with code $LASTEXITCODE"
}

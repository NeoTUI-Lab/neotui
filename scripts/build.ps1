# Build script for NeoTUI workspace
# Usage: .\scripts\build.ps1

Write-Host "Building NeoTUI workspace..." -ForegroundColor Green

cargo build --workspace

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!" -ForegroundColor Green
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Format script for NeoTUI workspace
# Usage: .\scripts\fmt.ps1

Write-Host "Formatting NeoTUI workspace..." -ForegroundColor Green

cargo fmt --all

if ($LASTEXITCODE -eq 0) {
    Write-Host "Formatting complete!" -ForegroundColor Green
} else {
    Write-Host "Formatting failed!" -ForegroundColor Red
    exit 1
}

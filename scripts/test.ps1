# Test script for NeoTUI workspace
# Usage: .\scripts\test.ps1

Write-Host "Running tests for NeoTUI workspace..." -ForegroundColor Green

cargo test --workspace

if ($LASTEXITCODE -eq 0) {
    Write-Host "Tests passed!" -ForegroundColor Green
} else {
    Write-Host "Tests failed!" -ForegroundColor Red
    exit 1
}

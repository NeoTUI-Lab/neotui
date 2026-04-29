# Lint script for NeoTUI workspace
# Usage: .\scripts\lint.ps1

Write-Host "Linting NeoTUI workspace..." -ForegroundColor Green

cargo clippy --workspace -- -D warnings

if ($LASTEXITCODE -eq 0) {
    Write-Host "Linting passed!" -ForegroundColor Green
} else {
    Write-Host "Linting failed!" -ForegroundColor Red
    exit 1
}

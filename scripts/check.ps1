# Check script for NeoTUI DSL validation
# Usage: .\scripts\check.ps1 <file>

param(
    [Parameter(Mandatory=$true)]
    [string]$File
)

Write-Host "Checking DSL file: $File" -ForegroundColor Green

if (-not (Test-Path $File)) {
    Write-Host "File not found: $File" -ForegroundColor Red
    exit 1
}

cargo run -p neotui-cli -- check $File

if ($LASTEXITCODE -eq 0) {
    Write-Host "DSL validation passed!" -ForegroundColor Green
} else {
    Write-Host "DSL validation failed!" -ForegroundColor Red
    exit 1
}

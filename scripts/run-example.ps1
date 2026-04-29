# Run example script for NeoTUI
# Usage: .\scripts\run-example.ps1 <example-file>

param(
    [Parameter(Mandatory=$true)]
    [string]$ExampleFile
)

Write-Host "Running example: $ExampleFile" -ForegroundColor Green

if (-not (Test-Path $ExampleFile)) {
    Write-Host "Example file not found: $ExampleFile" -ForegroundColor Red
    exit 1
}

cargo run -p neotui-cli -- run $ExampleFile

if ($LASTEXITCODE -eq 0) {
    Write-Host "Example executed successfully!" -ForegroundColor Green
} else {
    Write-Host "Example execution failed!" -ForegroundColor Red
    exit 1
}

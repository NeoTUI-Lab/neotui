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

# TODO: Implement actual DSL validation
# For now, just check if file exists and has valid TOML/JSON/YAML extension
$extension = [System.IO.Path]::GetExtension($File)

if ($extension -notin @('.toml', '.json', '.yaml', '.yml')) {
    Write-Host "Unsupported file extension: $extension" -ForegroundColor Red
    Write-Host "Supported extensions: .toml, .json, .yaml, .yml" -ForegroundColor Yellow
    exit 1
}

Write-Host "File extension is valid: $extension" -ForegroundColor Green
Write-Host "DSL validation not yet implemented - file structure check passed" -ForegroundColor Yellow

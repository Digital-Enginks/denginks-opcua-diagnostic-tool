# DENGINKS OPC-UA Diagnostic Tool - Build Script
# Creates a portable distribution package

param(
    [switch]$Release,
    [switch]$Package
)

$ErrorActionPreference = "Stop"

Write-Host "================================" -ForegroundColor Cyan
Write-Host "DENGINKS OPC-UA Diagnostic Tool" -ForegroundColor Cyan
Write-Host "Build Script v1.0" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Build
if ($Release) {
    Write-Host "[1/3] Building release version..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "Build successful!" -ForegroundColor Green
} else {
    Write-Host "[1/3] Skipping build (use -Release to build)" -ForegroundColor Gray
}

$distDir = ".\dist"
$distName = "denginks-opcua-diagnostic-portable"
$distPath = "$distDir\$distName"

if ($Package) {
    Write-Host "[2/3] Creating distribution package..." -ForegroundColor Yellow
    
    if (Test-Path $distPath) {
        Remove-Item $distPath -Recurse -Force
    }
    New-Item -ItemType Directory -Path $distPath -Force | Out-Null
    
    $exePath = ".\target\release\denginks-opcua-diagnostic.exe"
    if (-not (Test-Path $exePath)) {
        Write-Host "Executable not found! Run with -Release first." -ForegroundColor Red
        exit 1
    }
    Copy-Item $exePath "$distPath\"
    
    $mesaDll = ".\resources\mesa\opengl32.dll"
    if (Test-Path $mesaDll) {
        Copy-Item $mesaDll "$distPath\"
        Write-Host "  - Included Mesa3D opengl32.dll for software rendering" -ForegroundColor Gray
    } else {
        Write-Host "  - WARNING: Mesa3D opengl32.dll not found. Software rendering won't work on systems without GPU." -ForegroundColor Yellow
    }
    
    # Create README
    $readme = @"
# DENGINKS OPC-UA Diagnostic Tool
Portable Edition

## Usage
Simply run `denginks-opcua-diagnostic.exe`

## Graphics Compatibility
This package includes Mesa3D's opengl32.dll for software rendering.
This ensures the application works on:
- Windows Server 2012 R2+
- Virtual Machines without GPU acceleration
- Remote Desktop sessions
- Any system without modern graphics drivers

## Files
- denginks-opcua-diagnostic.exe - Main application
- opengl32.dll - Mesa3D software OpenGL renderer (optional but recommended)
- diagnostic.log - Created at runtime with debug information

## Troubleshooting
If the application fails to start:
1. Check diagnostic.log for error details
2. Ensure opengl32.dll is in the same folder as the .exe
3. Try running as Administrator

## Support
https://github.com/digital-enginks/denginks-opcua-diagnostic-tool
"@
    $readme | Out-File -FilePath "$distPath\README.txt" -Encoding utf8
    
    Write-Host "Distribution created at: $distPath" -ForegroundColor Green
    
    # Create ZIP archive
    Write-Host "[3/3] Creating ZIP archive..." -ForegroundColor Yellow
    $zipPath = "$distDir\$distName.zip"
    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
    Compress-Archive -Path "$distPath\*" -DestinationPath $zipPath -Force
    
    $zipSize = (Get-Item $zipPath).Length / 1MB
    Write-Host "ZIP archive created: $zipPath ($([math]::Round($zipSize, 2)) MB)" -ForegroundColor Green
} else {
    Write-Host "[2/3] Skipping package (use -Package to create distribution)" -ForegroundColor Gray
    Write-Host "[3/3] Skipping ZIP (use -Package to create distribution)" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green
Write-Host ""
Write-Host "Usage examples:" -ForegroundColor Cyan
Write-Host "  .\build.ps1 -Release           # Build release version only"
Write-Host "  .\build.ps1 -Release -Package  # Build and create distribution"
Write-Host "  .\build.ps1 -Package           # Create distribution (requires previous build)"

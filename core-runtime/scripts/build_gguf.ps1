# GGUF Backend Build Script
# This script properly sets up environment variables for building the GGUF backend

Write-Host "=== GGUF Backend Build Script ===" -ForegroundColor Cyan
Write-Host ""

# Check if LLVM is installed
$llvmPath = "C:\Program Files\llvm15.0.7\bin"
if (-not (Test-Path $llvmPath)) {
    Write-Host "ERROR: LLVM not found at $llvmPath" -ForegroundColor Red
    Write-Host "Please install LLVM first." -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] LLVM found at: $llvmPath" -ForegroundColor Green

# Check if libclang.dll exists
$libclangPath = Join-Path $llvmPath "libclang.dll"
if (-not (Test-Path $libclangPath)) {
    Write-Host "ERROR: libclang.dll not found at $libclangPath" -ForegroundColor Red
    Write-Host "Required files:" -ForegroundColor Yellow
    Get-ChildItem -Path $llvmPath -Filter "*.dll" | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor White
    }
    exit 1
}

Write-Host "[OK] libclang.dll found" -ForegroundColor Green

# Set environment variables
$env:LIBCLANG_PATH = $llvmPath
Write-Host "[OK] LIBCLANG_PATH set to: $llvmPath" -ForegroundColor Green

# Add LLVM to PATH if not already there
$currentPath = $env:PATH
if ($currentPath -notlike "*llvm*bin*") {
    $env:PATH = "$llvmPath;$currentPath"
    Write-Host "[OK] LLVM added to PATH" -ForegroundColor Green
} else {
    Write-Host "[OK] LLVM already in PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "Starting GGUF backend build..." -ForegroundColor Cyan
Write-Host ""

# Build the project
Set-Location -Path "g:\MythologIQ\CORE\core-runtime"

try {
    cargo build --features gguf
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host ""
        Write-Host "=== Build Successful ===" -ForegroundColor Green
        Write-Host "[OK] GGUF backend built successfully" -ForegroundColor Green
    } else {
        Write-Host ""
        Write-Host "=== Build Failed ===" -ForegroundColor Red
        Write-Host "Exit code: $exitCode" -ForegroundColor Yellow
        exit $exitCode
    }
} catch {
    Write-Host ""
    Write-Host "=== Build Error ===" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

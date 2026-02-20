# Run performance benchmark with VS2022 developer environment

# Set environment variables
$env:LIBCLANG_PATH = "C:\Program Files\llvm15.0.7\bin"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"

# Navigate to project
Set-Location "G:\MythologIQ\CORE\core-runtime"

Write-Host "`n=== GG-CORE Performance Benchmark ===" -ForegroundColor Cyan

# List available tests in e2e_model_test
Write-Host "Available tests:"
cargo test --features gguf --test e2e_model_test -- --list 2>&1 | Select-String "test"

Write-Host "`nRunning benchmark test (release mode)..."
cargo test --features gguf --release --test e2e_model_test e2e_performance -- --nocapture

# Run E2E test with VS2022 developer environment

# Set environment variables
$env:LIBCLANG_PATH = "C:\Program Files\llvm15.0.7\bin"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"

# Show current settings
Write-Host "LIBCLANG_PATH: $env:LIBCLANG_PATH"
Write-Host "CMAKE_GENERATOR: $env:CMAKE_GENERATOR"

# Navigate to project
Set-Location "G:\MythologIQ\CORE\core-runtime"

# Clean test artifacts and rebuild
Write-Host "`nCleaning test artifacts..."
Remove-Item -Path "target\x86_64-pc-windows-msvc\debug\deps\e2e_model_test*" -Force -ErrorAction SilentlyContinue

# Run all E2E tests with fresh build
Write-Host "`nRunning E2E tests (batch, streaming, chat, benchmark)..."
cargo test --features gguf --test e2e_model_test -- --nocapture

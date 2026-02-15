# Tier 1 Test Models Download Script
# This script downloads the required test models for Tier 1 validation

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Tier 1 Test Models Download Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Define model directories
$GGUFDir = Join-Path $ProjectRoot "fixtures\models\gguf"
$ONNXDir = Join-Path $ProjectRoot "fixtures\models\onnx"

Write-Host "Project Root: $ProjectRoot" -ForegroundColor Gray
Write-Host "GGUF Models Directory: $GGUFDir" -ForegroundColor Gray
Write-Host "ONNX Models Directory: $ONNXDir" -ForegroundColor Gray
Write-Host ""

# Create directories if they don't exist
Write-Host "Creating model directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $GGUFDir | Out-Null
New-Item -ItemType Directory -Force -Path $ONNXDir | Out-Null
Write-Host "✓ Directories created" -ForegroundColor Green
Write-Host ""

# Download phi3-mini-q4km.gguf
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Downloading phi3-mini-q4km.gguf..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Source: Hugging Face - Microsoft Phi-3" -ForegroundColor Gray
Write-Host "Size: ~2.3 GB" -ForegroundColor Gray
Write-Host "This may take several minutes..." -ForegroundColor Yellow
Write-Host ""

$GGUFUrl = "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf"
$GGUFPath = Join-Path $GGUFDir "phi3-mini-q4km.gguf"

try {
    Invoke-WebRequest -Uri $GGUFUrl -OutFile $GGUFPath -UseBasicParsing
    $FileSize = (Get-Item $GGUFPath).Length / 1GB
    Write-Host "✓ phi3-mini-q4km.gguf downloaded successfully" -ForegroundColor Green
    Write-Host "  Size: $([math]::Round($FileSize, 2)) GB" -ForegroundColor Gray
} catch {
    Write-Host "✗ Failed to download phi3-mini-q4km.gguf" -ForegroundColor Red
    Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Manual download instructions:" -ForegroundColor Yellow
    Write-Host "  1. Visit: $GGUFUrl" -ForegroundColor Gray
    Write-Host "  2. Download the file" -ForegroundColor Gray
    Write-Host "  3. Save to: $GGUFPath" -ForegroundColor Gray
}
Write-Host ""

# ONNX Models - Manual Download Required
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "ONNX Models (Manual Download Required)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "tinybert-classifier.onnx" -ForegroundColor Yellow
Write-Host "  Purpose: Classification testing" -ForegroundColor Gray
Write-Host "  Target Path: $ONNXDir\tinybert-classifier.onnx" -ForegroundColor Gray
Write-Host ""
Write-Host "Options:" -ForegroundColor Gray
Write-Host "  1. Download from Hugging Face:" -ForegroundColor Gray
Write-Host "     https://huggingface.co/models?search=tinybert+onnx" -ForegroundColor Cyan
Write-Host ""
Write-Host "  2. Convert from PyTorch:" -ForegroundColor Gray
Write-Host "     ```python" -ForegroundColor Gray
Write-Host "     import torch" -ForegroundColor Gray
Write-Host "     from transformers import AutoModelForSequenceClassification" -ForegroundColor Gray
Write-Host "     model = AutoModelForSequenceClassification.from_pretrained('prajjwal1/bert-tiny')" -ForegroundColor Gray
Write-Host "     dummy_input = torch.randint(0, 1000, (1, 128))" -ForegroundColor Gray
Write-Host "     torch.onnx.export(model, dummy_input, 'tinybert-classifier.onnx')" -ForegroundColor Gray
Write-Host "     ```" -ForegroundColor Gray
Write-Host ""

Write-Host "minilm-embedder.onnx" -ForegroundColor Yellow
Write-Host "  Purpose: Embedding testing" -ForegroundColor Gray
Write-Host "  Target Path: $ONNXDir\minilm-embedder.onnx" -ForegroundColor Gray
Write-Host ""
Write-Host "Options:" -ForegroundColor Gray
Write-Host "  1. Download from Hugging Face:" -ForegroundColor Gray
Write-Host "     https://huggingface.co/models?search=minilm+onnx" -ForegroundColor Gray
Write-Host ""
Write-Host "  2. Convert from PyTorch:" -ForegroundColor Gray
Write-Host "     ```python" -ForegroundColor Gray
Write-Host "     import torch" -ForegroundColor Gray
Write-Host "     from sentence_transformers import SentenceTransformer" -ForegroundColor Gray
Write-Host "     model = SentenceTransformer('all-MiniLM-L6-v2')" -ForegroundColor Gray
Write-Host "     dummy_input = torch.randint(0, 1000, (1, 128))" -ForegroundColor Gray
Write-Host "     torch.onnx.export(model, dummy_input, 'minilm-embedder.onnx')" -ForegroundColor Gray
Write-Host "     ```" -ForegroundColor Gray
Write-Host ""

# Summary
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Download Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$GGUFExists = Test-Path $GGUFPath
$TinyBertExists = Test-Path (Join-Path $ONNXDir "tinybert-classifier.onnx")
$MiniLMExists = Test-Path (Join-Path $ONNXDir "minilm-embedder.onnx")

Write-Host "Model Status:" -ForegroundColor Yellow
Write-Host "  phi3-mini-q4km.gguf: $(if ($GGUFExists) { '✓ Present' } else { '✗ Missing' })" -ForegroundColor $(if ($GGUFExists) { 'Green' } else { 'Red' })
Write-Host "  tinybert-classifier.onnx: $(if ($TinyBertExists) { '✓ Present' } else { '✗ Missing' })" -ForegroundColor $(if ($TinyBertExists) { 'Green' } else { 'Red' })
Write-Host "  minilm-embedder.onnx: $(if ($MiniLMExists) { '✓ Present' } else { '✗ Missing' })" -ForegroundColor $(if ($MiniLMExists) { 'Green' } else { 'Red' })
Write-Host ""

if (-not ($GGUFExists -and $TinyBertExists -and $MiniLMExists)) {
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. Download missing ONNX models manually" -ForegroundColor Gray
    Write-Host "  2. Place them in: $ONNXDir" -ForegroundColor Gray
    Write-Host "  3. Re-run this script to verify" -ForegroundColor Gray
    Write-Host ""
    Write-Host "For detailed instructions, see: TIER1_SETUP_GUIDE.md" -ForegroundColor Cyan
} else {
    Write-Host "✓ All models downloaded successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. Install build dependencies (protoc, libclang)" -ForegroundColor Gray
    Write-Host "  2. Build: cargo build --features onnx,gguf" -ForegroundColor Gray
    Write-Host "  3. Run tests: cargo test --features onnx,gguf" -ForegroundColor Gray
    Write-Host "  4. Run benchmarks: cargo bench --features onnx,gguf" -ForegroundColor Gray
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Script Complete" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

#!/bin/bash  
  
# collect_baseline.sh - Collect baseline metrics for regression detection  
  
set -e  
  
  
cd core-runtime  
  
  
# Run benchmarks with baseline collection  
  
cargo bench --features onnx,gguf,benchmarks  
  
  
echo "=========================================="  
  
echo "Baseline Collection Complete"  
  

  
#!/bin/bash  
  
# run_security.sh - Run security validation tests  
  
  
set -e  
  
  
  
  
echo "=========================================="  
  
echo "COREFORGE CORE Runtime - Security Tests"  
  
echo "=========================================="  
  
  
echo ""  
  
cd core-runtime  
  
  
# Run security tests only  
  
cargo test security  
  
  
  
  
echo ""  
  
echo "=========================================="  
  
echo "Security Tests Complete"  
  
echo "==========================================" 

# Quickstart: 10 Minutes to First Inference

**Target Audience:** DevOps Engineers, SREs, ML Engineers  
**Prerequisites:** Basic Kubernetes knowledge  
**Time Required:** ~10 minutes

---

## Overview

This guide will get you from zero to running your first LLM inference in approximately 10 minutes. By the end, you'll have:

- A running GG-CORE deployment
- A loaded model ready for inference
- A successful test inference

---

## Prerequisites Check (2 minutes)

### Required Tools

| Tool       | Version | Check Command              |
| ---------- | ------- | -------------------------- |
| Kubernetes | 1.28+   | `kubectl version --short`  |
| Helm       | 3.12+   | `helm version --short`     |
| kubectl    | Latest  | `kubectl version --client` |

### Required Resources

| Resource  | Minimum  | Recommended      |
| --------- | -------- | ---------------- |
| CPU Cores | 4        | 8+               |
| Memory    | 16GB     | 32GB+            |
| GPU       | Optional | NVIDIA A100/H100 |
| Storage   | 50GB     | 100GB+ SSD       |

### Quick Prerequisite Check

```bash
# Check Kubernetes connection
kubectl cluster-info

# Check node resources
kubectl describe nodes | grep -A 5 "Allocated resources"

# Check GPU availability (if applicable)
kubectl describe nodes | grep nvidia.com/gpu
```

---

## Step 1: Add Helm Repository (1 minute)

```bash
# Add the GG-CORE Helm repository
helm repo add GG-CORE https://charts.GG-CORE.io

# Update repository
helm repo update

# Verify chart is available
helm search repo GG-CORE
```

**Expected Output:**

```
NAME                    CHART VERSION   APP VERSION     DESCRIPTION
GG-CORE/GG-CORE 0.7.0           0.7.0           Secure LLM Inference Runtime
```

---

## Step 2: Create Namespace (30 seconds)

```bash
# Create dedicated namespace
kubectl create namespace GG-CORE

# Set as default for subsequent commands
kubectl config set-context --current --namespace=GG-CORE
```

---

## Step 3: Deploy GG-CORE (3 minutes)

### Option A: Quick Development Deployment (CPU-only)

```bash
# Deploy with minimal configuration for evaluation
helm install GG-CORE GG-CORE/GG-CORE \
  --namespace GG-CORE \
  --set replicaCount=1 \
  --set resources.limits.cpu=2 \
  --set resources.limits.memory=4Gi \
  --set model.enabled=true \
  --set model.name="llama-2-7b-chat" \
  --set model.quantization="q4_0"
```

### Option B: Single GPU Deployment

```bash
# Deploy with GPU support
helm install GG-CORE GG-CORE/GG-CORE \
  --namespace GG-CORE \
  --set replicaCount=1 \
  --set resources.limits.nvidia.com/gpu=1 \
  --set model.enabled=true \
  --set model.name="llama-2-7b-chat"
```

### Option C: Using Example Values File

```bash
# Download example values file
curl -O https://raw.githubusercontent.com/GG-CORE/charts/main/examples/values-dev.yaml

# Deploy with example values
helm install GG-CORE GG-CORE/GG-CORE \
  --namespace GG-CORE \
  -f values-dev.yaml
```

---

## Step 4: Verify Deployment (2 minutes)

### Check Pod Status

```bash
# Watch pods start
kubectl get pods -w

# Wait for pod to be ready (Ctrl+C when ready)
```

**Expected Output:**

```
NAME                           READY   STATUS    RESTARTS   AGE
GG-CORE-6f8b9c4d-xyz12     0/1     Pending   0          0s
GG-CORE-6f8b9c4d-xyz12     0/1     ContainerCreating   0   2s
GG-CORE-6f8b9c4d-xyz12     1/1     Running   0          45s
```

### Check Deployment Health

```bash
# Check deployment status
kubectl get deployment GG-CORE

# Check service
kubectl get svc GG-CORE

# Check model loading
kubectl logs -l app.kubernetes.io/name=GG-CORE --tail=50 | grep -i model
```

### Run Verification Command

```bash
# If GG-CORE CLI is installed
GG-CORE verify

# Or use kubectl
kubectl exec -it deployment/GG-CORE -- GG-CORE-verify
```

**Expected Output:**

```
✓ Pod is running
✓ Service is accessible
✓ Model loaded: llama-2-7b-chat
✓ Health check passed
✓ Ready for inference
```

---

## Step 5: Run First Inference (2 minutes)

### Port Forward (if not using LoadBalancer/Ingress)

```bash
# Forward local port to service
kubectl port-forward svc/GG-CORE 8080:8080 &

# Wait for port forward to establish
sleep 2
```

### Test Inference

```bash
# Simple completion request
curl -X POST http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b-chat",
    "prompt": "Explain quantum computing in one sentence.",
    "max_tokens": 50,
    "temperature": 0.7
  }'
```

**Expected Response:**

```json
{
  "id": "cmpl-abc123",
  "object": "text_completion",
  "created": 1708123456,
  "model": "llama-2-7b-chat",
  "choices": [
    {
      "text": "Quantum computing uses quantum mechanical phenomena like superposition and entanglement to process information in ways that classical computers cannot, potentially solving certain problems exponentially faster.",
      "index": 0,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 8,
    "completion_tokens": 32,
    "total_tokens": 40
  }
}
```

### Streaming Inference

```bash
# Streaming completion request
curl -X POST http://localhost:8080/v1/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-2-7b-chat",
    "prompt": "Write a haiku about AI:",
    "max_tokens": 30,
    "stream": true
  }'
```

---

## Step 6: Explore Further (Optional)

### Check Available Models

```bash
# List loaded models
curl http://localhost:8080/v1/models

# Or via kubectl
kubectl exec -it deployment/GG-CORE -- GG-CORE models list
```

### View Metrics

```bash
# Prometheus metrics endpoint
curl http://localhost:8080/metrics

# Key metrics to check
curl http://localhost:8080/metrics | grep -E "veritas_(requests|latency|tokens)"
```

### Check Logs

```bash
# View recent logs
kubectl logs -l app.kubernetes.io/name=GG-CORE --tail=100

# Follow logs
kubectl logs -f -l app.kubernetes.io/name=GG-CORE
```

---

## Troubleshooting

### Pod Stuck in Pending

```bash
# Check events
kubectl describe pod -l app.kubernetes.io/name=GG-CORE

# Common causes:
# - Insufficient resources: Reduce resource requests
# - GPU not available: Use CPU-only deployment
# - PVC not binding: Check storage class
```

### Model Not Loading

```bash
# Check model loading logs
kubectl logs -l app.kubernetes.io/name=GG-CORE | grep -i "model\|error"

# Common causes:
# - Insufficient memory: Use quantized model
# - Model not found: Check model name
# - Download timeout: Check network connectivity
```

### Connection Refused

```bash
# Check service
kubectl get svc GG-CORE
kubectl describe svc GG-CORE

# Check endpoints
kubectl get endpoints GG-CORE

# Verify port forward is running
ps aux | grep port-forward
```

### GPU Not Detected

```bash
# Check GPU resources
kubectl describe nodes | grep -A 10 "nvidia.com/gpu"

# Check GPU operator (if using NVIDIA)
kubectl get pods -n gpu-operator

# Verify GPU driver
kubectl exec -it deployment/GG-CORE -- nvidia-smi
```

---

## Cleanup

```bash
# Uninstall Helm release
helm uninstall GG-CORE --namespace GG-CORE

# Delete namespace
kubectl delete namespace GG-CORE

# Remove Helm repository (optional)
helm repo remove GG-CORE
```

---

## Next Steps

1. **Configuration:** See [values.yaml examples](../../k8s/helm/GG-CORE/examples/) for production configurations
2. **Deployment Strategies:** Read [ADR-006](../architecture/ADR-006-DEPLOYMENT-STRATEGIES.md) for canary/blue-green deployments
3. **Security:** Review [Security Posture Baseline](../security/SECURITY_POSTURE_BASELINE.md) for production hardening
4. **Operations:** Check [Deployment Troubleshooting](./DEPLOYMENT_TROUBLESHOOTING.md) for common issues
5. **Monitoring:** Import Grafana dashboards for observability

---

## Getting Help

| Resource           | Link                                       |
| ------------------ | ------------------------------------------ |
| Documentation      | https://docs.GG-CORE.io                |
| GitHub Issues      | https://github.com/GG-CORE/core/issues |
| Community Slack    | https://slack.GG-CORE.io               |
| Enterprise Support | support@GG-CORE.io                     |

---

## Quick Reference

### Essential Commands

```bash
# Install
helm install GG-CORE GG-CORE/GG-CORE -n GG-CORE

# Upgrade
helm upgrade GG-CORE GG-CORE/GG-CORE -n GG-CORE

# Rollback
helm rollback GG-CORE -n GG-CORE

# Uninstall
helm uninstall GG-CORE -n GG-CORE

# Check status
kubectl get all -n GG-CORE

# View logs
kubectl logs -f -l app.kubernetes.io/name=GG-CORE -n GG-CORE

# Port forward
kubectl port-forward svc/GG-CORE 8080:8080 -n GG-CORE

# Test inference
curl http://localhost:8080/v1/models
```

### Example values.yaml

```yaml
# Minimal development configuration
replicaCount: 1

resources:
  limits:
    cpu: 2
    memory: 4Gi
  requests:
    cpu: 1
    memory: 2Gi

model:
  enabled: true
  name: "llama-2-7b-chat"
  quantization: "q4_0"

service:
  type: ClusterIP
  port: 8080

# Disable GPU for CPU-only
gpu:
  enabled: false
```

---

**Congratulations!** You've successfully deployed GG-CORE and run your first inference. Welcome to secure, production-ready LLM inference!

# ADR-006: Deployment Strategies

**Status:** Accepted  
**Date:** 2026-02-18  
**Decision Makers:** Architecture Team, DevOps Team  
**Consulted:** Security Team, Operations Team

---

## Context

GG-CORE v0.6.0 introduces production-grade deployment capabilities. The system requires deployment strategies that:

1. Minimize downtime during updates
2. Enable safe rollback on failure detection
3. Support progressive traffic shifting for risk mitigation
4. Integrate with existing Kubernetes infrastructure
5. Provide clear operator visibility and control

The deployment system must handle LLM inference workloads with specific characteristics:

- Long-lived connections for streaming inference
- GPU resource constraints requiring careful capacity planning
- Model loading latency affecting startup time
- Memory-intensive workloads sensitive to over-provisioning

---

## Decision

We will implement **two complementary deployment strategies**:

### 1. Canary Deployments (Progressive Rollout)

- **Use Case:** Gradual rollout with automated analysis and rollback
- **Implementation:** Custom CanaryController (canary.rs) with Kubernetes CRD
- **Analysis:** Automated metric analysis with configurable thresholds
- **Rollback:** Automatic on threshold violation

### 2. Blue-Green Deployments (Instant Switch)

- **Use Case:** Instant traffic switching with verified readiness
- **Implementation:** Dual Kubernetes Services with Helm template switching
- **Verification:** Health checks before traffic switch
- **Rollback:** Instant switch back to previous version

---

## Rationale

### Why Both Strategies?

| Factor             | Canary                           | Blue-Green                  |
| ------------------ | -------------------------------- | --------------------------- |
| **Risk Tolerance** | Lower risk, progressive exposure | Higher risk, instant switch |
| **Rollback Speed** | Minutes (progressive)            | Seconds (instant)           |
| **Resource Cost**  | Single environment + canary pods | Double environment          |
| **Analysis Depth** | Deep metric analysis             | Binary health check         |
| **Use Case**       | Production updates               | Hotfixes, rollbacks         |

### Why Custom CanaryController?

1. **LLM-Specific Metrics:** Generic controllers don't understand token latency, generation throughput, or memory pressure specific to inference workloads.

2. **Integration Requirements:** Tight integration with existing metrics pipeline (metrics.rs) and security monitoring.

3. **Control Requirements:** Fine-grained control over analysis thresholds (thresholds.rs) and rollback behavior.

4. **Dependency Management:** Avoiding external dependencies (Argo Rollouts, Flagger) reduces operational complexity.

### Why Helm-Based Blue-Green?

1. **Simplicity:** Operators can understand and audit the deployment without service mesh expertise.

2. **GitOps Compatible:** Traffic switching is a values.yaml change, fully auditable in Git.

3. **Resource Efficiency:** No service mesh overhead (approximately 500MB per node avoided).

4. **Verification Built-in:** Health checks in bluegreen-service.yaml ensure readiness before traffic switch.

---

## Detailed Design

### Canary Deployment Flow

```
                    Start Canary
                         |
                         v
    +--------------------+--------------------+
    |                    |                    |
    v                    v                    v
[10% Traffic]      [25% Traffic]      [50% Traffic]
    |                    |                    |
    v                    v                    v
[Analyze]           [Analyze]           [Analyze]
    |                    |                    |
    +--------+-----------+--------+-----------+
             |                    |
             v                    v
      [Pass: Continue]     [Fail: Rollback]
             |                    |
             v                    v
      [100% Traffic]       [Revert to Stable]
             |
             v
        [Complete]
```

### Canary Analysis Thresholds

| Metric           | Threshold      | Window |
| ---------------- | -------------- | ------ |
| Error Rate       | < 1%           | 60s    |
| P50 Latency      | < 100ms        | 60s    |
| P99 Latency      | < 500ms        | 60s    |
| Token Throughput | > 95% baseline | 60s    |
| Memory Pressure  | < 90%          | 60s    |
| GPU Utilization  | < 95%          | 60s    |

### Blue-Green Deployment Flow

```
    [Current: Blue] ----------+
         |                    |
         v                    v
    [Deploy Green]      [Verify Green]
         |                    |
         v                    v
    [Green Ready]        [Health Checks]
         |                    |
         v                    v
    [Switch Traffic] --> [Traffic -> Green]
         |
         v
    [Monitor Green]
         |
    +----+----+
    |         |
    v         v
 [Stable]  [Issue: Rollback]
    |         |
    v         v
[Retire Blue] [Traffic -> Blue]
```

---

## Implementation

### Canary Controller (canary.rs)

```rust
pub struct CanaryController {
    analysis_thresholds: AnalysisThresholds,
    metrics: DeploymentMetrics,
    rollback_manager: RollbackManager,
}

impl CanaryController {
    pub async fn reconcile(&self, canary: &Canary) -> Result<CanaryStatus> {
        // 1. Check current phase
        // 2. Collect metrics for analysis window
        // 3. Evaluate against thresholds
        // 4. Advance phase or trigger rollback
        // 5. Update status
    }
}
```

### Blue-Green Service Template (bluegreen-service.yaml)

```yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "GG-CORE.fullname" . }}-active
spec:
  selector:
    app.kubernetes.io/name: {{ include "GG-CORE.name" . }}
    app.kubernetes.io/version: {{ .Values.activeVersion }}  # "blue" or "green"
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
```

---

## Alternatives Considered

### Alternative 1: Argo Rollouts

**Pros:**

- Battle-tested, feature-rich
- Active community
- Advanced traffic management

**Cons:**

- Additional dependency
- Learning curve for operators
- Less control over LLM-specific analysis

**Decision:** Rejected due to dependency overhead and lack of LLM-specific analysis.

### Alternative 2: Service Mesh (Istio)

**Pros:**

- Advanced traffic management
- Built-in observability
- Industry standard

**Cons:**

- High resource overhead (~500MB/node)
- Significant complexity
- Requires Istio expertise

**Decision:** Rejected due to resource overhead and complexity for our use case.

### Alternative 3: Kubernetes Native Rolling Update

**Pros:**

- No additional components
- Simple configuration
- Native behavior

**Cons:**

- No automated analysis
- No progressive traffic shifting
- Rollback requires manual intervention

**Decision:** Rejected due to lack of automated analysis and rollback capabilities.

---

## Consequences

### Positive

1. **Risk Mitigation:** Canary deployments catch issues before full exposure.
2. **Fast Rollback:** Blue-green enables instant rollback for critical issues.
3. **Operator Control:** Clear visibility and control through CRDs and Helm values.
4. **No External Dependencies:** Core functionality works without service mesh or additional controllers.
5. **LLM-Specific Analysis:** Custom thresholds for inference workload characteristics.

### Negative

1. **Development Effort:** Custom controller requires development and maintenance.
2. **Resource Cost:** Blue-green requires double environment resources.
3. **Learning Curve:** Operators must understand both strategies.
4. **Limited Traffic Management:** No advanced features like traffic mirroring or weighted routing without service mesh.

### Mitigations

1. **Documentation:** Comprehensive runbooks in DEPLOYMENT_TROUBLESHOOTING.md.
2. **Testing:** Extensive test coverage (canary_deployment_test.rs, bluegreen_deployment_test.rs).
3. **Chaos Testing:** Chaos tests validate rollback behavior (canary_chaos_test.rs, bluegreen_chaos_test.rs).
4. **Resource Planning:** Blue-green resource requirements documented in capacity planning.

---

## Migration Path

### Phase 1: v0.6.0 (Current)

- Canary CRD and controller deployed
- Blue-green Helm templates available
- Basic analysis thresholds configured
- Manual traffic switching for blue-green

### Phase 2: v0.6.1

- Automated blue-green switching with verification
- Enhanced canary analysis with ML-based anomaly detection
- Grafana dashboards for deployment visibility

### Phase 3: v0.7.0

- Service mesh integration option (optional)
- Traffic mirroring for pre-production testing
- Advanced deployment strategies (A/B testing)

---

## Monitoring and Observability

### Canary Metrics

| Metric                           | Description                  | Alert Threshold   |
| -------------------------------- | ---------------------------- | ----------------- |
| `gg_core_canary_progress`        | Current canary phase (0-100) | N/A               |
| `gg_core_canary_analysis_total`  | Total analysis runs          | N/A               |
| `gg_core_canary_analysis_failed` | Failed analysis runs         | > 0               |
| `gg_core_canary_rollback_total`  | Total rollbacks              | > 0 (investigate) |

### Blue-Green Metrics

| Metric                                       | Description      | Alert Threshold   |
| -------------------------------------------- | ---------------- | ----------------- |
| `gg_core_bluegreen_switch_total`             | Traffic switches | N/A               |
| `gg_core_bluegreen_switch_duration_seconds`  | Switch duration  | > 30s             |
| `gg_core_bluegreen_rollback_total`           | Rollbacks        | > 0 (investigate) |

---

## Security Considerations

1. **RBAC:** Canary controller requires limited RBAC permissions (documented in rbac.yaml).
2. **Audit Logging:** All deployment changes logged for compliance.
3. **Approval Gates:** Production deployments require approval (configurable).
4. **Rollback Authorization:** Rollback requires appropriate RBAC permissions.

---

## References

- [Canary CRD Definition](../k8s/crds/canary.yaml)
- [Environment CRD Definition](../k8s/crds/environment.yaml)
- [Canary Controller Implementation](../../core-runtime/src/deployment/canary.rs)
- [Deployment Metrics](../../core-runtime/src/deployment/metrics.rs)
- [Analysis Thresholds](../../core-runtime/src/deployment/thresholds.rs)
- [Deployment Troubleshooting Guide](../operations/DEPLOYMENT_TROUBLESHOOTING.md)
- [Chaos Runbook](../operations/CHAOS_RUNBOOK.md)

---

## Revision History

| Version | Date       | Author            | Changes     |
| ------- | ---------- | ----------------- | ----------- |
| 1.0.0   | 2026-02-18 | Architecture Team | Initial ADR |

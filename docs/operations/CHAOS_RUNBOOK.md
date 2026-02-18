# Chaos Engineering Runbook

**Version:** 1.0.0
**Last Updated:** 2026-02-18
**Status:** Active

This runbook documents chaos engineering practices for the Veritas SDR runtime, including blast radius analysis, recovery procedures, and pre-planned game day scenarios.

---

## Table of Contents

1. [Overview](#overview)
2. [Blast Radius Analysis](#blast-radius-analysis)
3. [Recovery Procedures](#recovery-procedures)
4. [Game Day Scenarios](#game-day-scenarios)
5. [Appendix: Chaos Test Reference](#appendix-chaos-test-reference)

---

## Overview

### Purpose

Chaos engineering validates system resilience through controlled failure injection. This runbook provides:

- **Blast radius analysis** for deployment failures
- **Recovery procedures** for common failure scenarios
- **Game day scenarios** for team training and system validation

### Scope

| Deployment Type | Coverage |
|----------------|----------|
| Canary Deployments | Instance crashes, error rate spikes, latency issues, metrics failures |
| Blue-Green Deployments | Traffic switch failures, state sync failures, resource exhaustion |

### Safety Principles

1. **Steady state hypothesis** defined before each experiment
2. **Blast radius** controlled and documented
3. **Automatic rollback** available within 30 seconds
4. **No production customer impact** (staging-first policy)
5. **Metrics collection** active during all experiments
6. **Learning captured** and improvements implemented

---

## Blast Radius Analysis

### Canary Deployment Failures

#### What Fails When Canary Fails?

| Failure Mode | Direct Impact | Indirect Impact | Blast Radius |
|--------------|---------------|-----------------|--------------|
| Canary pod crash | 10% traffic affected (default canary weight) | Health check alerts trigger | Low |
| Elevated error rate | Affected sessions receive errors | Metrics anomaly alerts | Low-Medium |
| Latency spike | Slow responses for canary traffic | SLO breach potential | Low |
| All canary replicas down | 10% traffic unavailable | Automatic rollback trigger | Medium |

**Containment Mechanisms:**
- Traffic automatically routes to control group on canary failure
- Rollback controller triggers after 3 consecutive failures
- Error rate threshold (5%) triggers automatic rollback
- P99 latency threshold (500ms) triggers automatic rollback

#### What Fails When Metrics System Fails?

| Failure Mode | Direct Impact | Indirect Impact | Blast Radius |
|--------------|---------------|-----------------|--------------|
| Metrics unavailable | No canary comparison data | Manual monitoring required | Low |
| Delayed metrics | Rollback decision delayed | Extended degradation window | Medium |
| Corrupted metrics | False positive/negative decisions | Incorrect rollback action | Medium-High |

**Safe Fallback Behavior:**
- No automatic rollback without confirmed degradation
- Alert on-call engineer for manual investigation
- Default to conservative (keep current state) when uncertain

### Blue-Green Deployment Failures

#### What Fails When Switch Fails?

| Failure Mode | Direct Impact | Indirect Impact | Blast Radius |
|--------------|---------------|-----------------|--------------|
| Switch interrupted | Partial traffic split | Session inconsistency | High |
| DNS propagation delay | Gradual transition (expected) | None (expected behavior) | None |
| New environment unhealthy | No traffic reaches new env | Rollback required | Medium |
| State sync failure | Sessions lost on new env | User session errors | High |

**Containment Mechanisms:**
- Double-switch prevention (only one switch in progress)
- Health check before completing switch
- Rollback available to previous environment
- Session state preserved on old environment

#### Maximum Impact Scenarios

| Scenario | Probability | Impact | Mitigation |
|----------|-------------|--------|------------|
| Canary + Control both fail | Very Low | 100% traffic affected | Regional failover, circuit breaker |
| Blue-Green switch with no rollback | Very Low | New env failures unrecoverable | Preserve old env until validation |
| Cascading failures | Low | Multiple system degradation | Blast radius isolation, circuit breakers |

---

## Recovery Procedures

### Manual Rollback Steps

#### Canary Rollback

**Symptoms:**
- Elevated error rates on canary variant
- Canary pods in CrashLoopBackOff
- P99 latency exceeding thresholds

**Procedure:**

```bash
# 1. Verify current traffic split
kubectl get configmap veritas-traffic-config -o yaml

# 2. Set traffic to 100% control
kubectl patch configmap veritas-traffic-config \
  --patch '{"data":{"control":"100","canary":"0"}}'

# 3. Verify traffic shift
kubectl logs -l app=veritas -c envoy --tail=100 | grep variant

# 4. Confirm canary pods are no longer receiving traffic
kubectl exec -it deploy/veritas-runtime -- curl localhost:9090/metrics | grep canary

# 5. Investigate root cause
kubectl logs -l variant=canary --previous
kubectl describe pod -l variant=canary
```

**Expected Duration:** < 2 minutes

#### Blue-Green Rollback

**Symptoms:**
- New environment failing health checks
- High error rates after switch
- Resource exhaustion on new environment

**Procedure:**

```bash
# 1. Identify current active environment
kubectl get service veritas-runtime -o jsonpath='{.spec.selector}'

# 2. Switch back to previous environment
kubectl patch service veritas-runtime \
  --patch '{"spec":{"selector":{"deployment":"blue"}}}'

# 3. Verify traffic routing
kubectl exec -it deploy/veritas-runtime-blue -- curl localhost:8080/health

# 4. Confirm old environment is healthy
kubectl get pods -l deployment=blue -o wide

# 5. Drain and investigate new environment
kubectl cordon -l deployment=green
kubectl logs -l deployment=green --all-containers
```

**Expected Duration:** < 3 minutes

### State Recovery Procedures

#### KV-Cache Recovery

**Symptoms:**
- Cache miss rate elevated after switch
- Slow inference responses (cold cache)

**Procedure:**

```bash
# 1. Check cache status
kubectl exec -it deploy/veritas-runtime -- ./cli cache-status

# 2. Trigger cache warm-up
kubectl exec -it deploy/veritas-runtime -- ./cli cache-warmup --model llama-7b

# 3. Monitor warm-up progress
kubectl logs deploy/veritas-runtime -f | grep "cache_warm"

# 4. Verify cache health
kubectl exec -it deploy/veritas-runtime -- ./cli cache-status
```

**Expected Duration:** 5-15 minutes (depends on model size)

#### Session State Recovery

**Symptoms:**
- Users experiencing session errors
- "Session not found" errors in logs

**Procedure:**

```bash
# 1. Identify affected sessions
kubectl logs deploy/veritas-runtime | grep "session_not_found" | cut -d' ' -f3 | sort -u

# 2. Check session store
kubectl exec -it deploy/veritas-runtime -- ./cli session-list

# 3. If using distributed sessions, verify connectivity
kubectl exec -it deploy/veritas-runtime -- ./cli session-store-health

# 4. For non-recoverable sessions, notify affected users
# (Graceful degradation - new sessions will work)
```

**Expected Duration:** Varies

### Escalation Paths

| Level | Criteria | Contact | Response Time |
|-------|----------|---------|---------------|
| L1 | Single component failure, auto-recovery works | On-call engineer | 15 minutes |
| L2 | Multiple component failure, manual recovery needed | Platform team | 30 minutes |
| L3 | Widespread outage, data loss potential | Engineering lead + Security | Immediate |

**Escalation Triggers:**
- Automatic rollback fails
- Multiple environments affected simultaneously
- Data corruption detected
- Security incident suspected

---

## Game Day Scenarios

### Pre-Planned Chaos Experiments

#### Scenario 1: Canary Pod Termination

**Objective:** Validate automatic failover when canary instances crash.

**Hypothesis:**
- Traffic will automatically route to control group
- No user-visible errors beyond in-flight requests
- Rollback triggers within 30 seconds

**Procedure:**
1. Deploy canary with 10% traffic
2. Send steady traffic load (100 RPS)
3. Kill canary pod: `kubectl delete pod -l variant=canary --force`
4. Observe metrics and rollback behavior
5. Verify control group handles 100% traffic

**Success Criteria:**
- [ ] Canary traffic drops to 0% within 10 seconds
- [ ] Control group latency increases < 10%
- [ ] No 5xx errors to users (except in-flight)
- [ ] Alert fires within 1 minute

**Rollback Trigger:** N/A (test validates automatic rollback)

**Duration:** 15 minutes

---

#### Scenario 2: Elevated Error Rate Injection

**Objective:** Validate error rate detection and rollback.

**Hypothesis:**
- Error rates above 5% trigger investigation
- Sustained errors trigger automatic rollback
- Metrics accurately reflect error state

**Procedure:**
1. Deploy canary with 10% traffic
2. Inject 20% error rate via fault injection
3. Monitor error rate metrics
4. Observe rollback controller behavior
5. Verify clean rollback to control

**Success Criteria:**
- [ ] Error rate detected within 1 minute
- [ ] Rollback triggers within 3 minutes
- [ ] Post-rollback error rate returns to baseline
- [ ] All failure events logged

**Rollback Trigger:** Manual if automatic fails after 5 minutes

**Duration:** 20 minutes

---

#### Scenario 3: Blue-Green Switch with Unhealthy Target

**Objective:** Validate health check prevents switch to unhealthy environment.

**Hypothesis:**
- Switch will not complete if target is unhealthy
- Traffic remains on current (healthy) environment
- Appropriate alerts fire

**Procedure:**
1. Deploy green environment with induced failure
2. Initiate blue-green switch
3. Observe switch blocked by health check
4. Verify blue environment continues serving
5. Fix green environment, retry switch

**Success Criteria:**
- [ ] Switch blocked by health check
- [ ] Zero traffic reaches unhealthy green
- [ ] Blue environment unaffected
- [ ] Health check failure logged with reason

**Rollback Trigger:** Restore blue-only routing if green receives traffic

**Duration:** 25 minutes

---

#### Scenario 4: Metrics System Outage

**Objective:** Validate safe behavior when metrics are unavailable.

**Hypothesis:**
- System continues operating without metrics
- No automatic actions taken without data
- Alert fires for metrics outage
- Manual investigation triggered

**Procedure:**
1. Deploy canary with 10% traffic
2. Disable metrics collection (network partition simulation)
3. Observe system behavior
4. Verify no automatic rollback (no data to trigger)
5. Restore metrics, verify data collection resumes

**Success Criteria:**
- [ ] No automatic rollback without metrics
- [ ] Traffic continues flowing normally
- [ ] Metrics outage alert fires within 2 minutes
- [ ] Metrics recovery is clean (no data gaps beyond outage)

**Rollback Trigger:** Manual if system becomes unstable

**Duration:** 20 minutes

---

#### Scenario 5: KV-Cache Corruption

**Objective:** Validate detection and recovery from cache corruption.

**Hypothesis:**
- Cache corruption is detected
- Graceful degradation to cold cache mode
- Recovery through cache invalidation and warm-up

**Procedure:**
1. Populate KV-cache with valid entries
2. Inject corruption (invalid data marker)
3. Observe cache read failures
4. Trigger cache invalidation
5. Warm cache with fresh data
6. Verify normal operation resumes

**Success Criteria:**
- [ ] Corruption detected on read
- [ ] Graceful fallback to uncached mode
- [ ] Invalidation completes without data loss
- [ ] Warm-up restores normal performance

**Rollback Trigger:** Restart runtime if cache cannot be invalidated

**Duration:** 30 minutes

---

#### Scenario 6: GPU Unavailability During Switch

**Objective:** Validate handling of GPU resource failures during blue-green switch.

**Hypothesis:**
- Health check detects GPU unavailability
- Switch is blocked or rolled back
- Fallback to CPU mode if configured

**Procedure:**
1. Deploy green environment with GPU requirement
2. Simulate GPU unavailability (driver fault)
3. Initiate blue-green switch
4. Observe switch behavior
5. Verify graceful handling

**Success Criteria:**
- [ ] GPU unavailability detected pre-switch or immediately
- [ ] Switch blocked or immediate rollback
- [ ] Blue environment continues serving
- [ ] Clear logging of GPU failure

**Rollback Trigger:** Emergency rollback if traffic reaches GPU-less green

**Duration:** 20 minutes

---

### Game Day Execution Checklist

**Before Game Day:**
- [ ] Schedule maintenance window (if production)
- [ ] Notify stakeholders
- [ ] Verify monitoring dashboards accessible
- [ ] Confirm rollback procedures tested
- [ ] Assign observation roles

**During Game Day:**
- [ ] Start recording/logging session
- [ ] Execute scenario per procedure
- [ ] Document observations in real-time
- [ ] Capture metrics snapshots
- [ ] Execute rollback if needed

**After Game Day:**
- [ ] Conduct post-mortem discussion
- [ ] Document findings
- [ ] Create improvement tickets
- [ ] Update this runbook with learnings
- [ ] Share results with team

### Observation Roles

| Role | Responsibility |
|------|---------------|
| Scenario Lead | Execute chaos injection, manage timeline |
| Metrics Observer | Monitor dashboards, capture screenshots |
| Log Observer | Watch logs, capture error patterns |
| Communication Lead | Update stakeholders, manage escalations |
| Scribe | Document observations, timeline, decisions |

---

## Appendix: Chaos Test Reference

### Test Files

| File | Description |
|------|-------------|
| `tests/chaos/canary_chaos_test.rs` | Canary deployment failure scenarios |
| `tests/chaos/bluegreen_chaos_test.rs` | Blue-green switch failure scenarios |
| `tests/chaos/mod.rs` | Module definition |

### Running Chaos Tests

```bash
# Run all chaos tests
cargo test --test '*chaos*'

# Run canary chaos tests only
cargo test canary_chaos

# Run blue-green chaos tests only
cargo test bluegreen_chaos

# Run with verbose output
cargo test --test '*chaos*' -- --nocapture
```

### Test Coverage Matrix

| Category | Test | Failure Injected |
|----------|------|------------------|
| Canary | `test_canary_pod_crash_during_rollout` | Pod crash |
| Canary | `test_canary_elevated_error_rate_triggers_rollback` | 30% error rate |
| Canary | `test_canary_latency_spike_triggers_rollback` | 1500ms latency |
| Canary | `test_automatic_rollback_within_threshold` | Consecutive failures |
| Canary | `test_metrics_unavailable_safe_fallback` | Metrics outage |
| Canary | `test_delayed_metrics_reporting` | Metrics delay |
| Canary | `test_corrupted_metrics_data` | Data corruption |
| Canary | `test_some_canary_replicas_failing` | Partial replica failure |
| Canary | `test_resource_exhaustion_on_canary` | OOM/CPU exhaustion |
| Blue-Green | `test_traffic_switch_interrupted_mid_execution` | Switch interruption |
| Blue-Green | `test_new_environment_unhealthy_after_switch` | Post-switch failure |
| Blue-Green | `test_dns_propagation_delay` | Network delay |
| Blue-Green | `test_cache_warming_failure` | Cache init failure |
| Blue-Green | `test_kv_cache_corruption` | Data corruption |
| Blue-Green | `test_kv_cache_sync_failure` | Sync failure |
| Blue-Green | `test_session_state_loss` | State loss |
| Blue-Green | `test_standby_environment_oom` | Memory exhaustion |
| Blue-Green | `test_disk_full_on_new_environment` | Disk full |
| Blue-Green | `test_gpu_unavailable_on_new_environment` | GPU failure |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-02-18 | Chaos Engineer | Initial version |

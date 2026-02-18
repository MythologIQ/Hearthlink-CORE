# SOC 2 Control Matrix

**Version:** 1.0.0  
**Assessment Date:** 2026-02-18  
**Compliance Level:** 91%  
**Trust Service Criteria:** Security (CC), Availability (A)

---

## Executive Summary

This document maps Veritas SDR's implemented controls to SOC 2 Trust Service Criteria. The current compliance level is **91%**, with 61 of 67 applicable controls fully implemented.

### Compliance Status Overview

| Category                         | Total Controls | Implemented | Partial | Not Implemented | Compliance % |
| -------------------------------- | -------------- | ----------- | ------- | --------------- | ------------ |
| CC6.1 - Logical Access           | 8              | 8           | 0       | 0               | 100%         |
| CC6.2 - Access Removal           | 4              | 4           | 0       | 0               | 100%         |
| CC6.3 - Access Authorization     | 6              | 6           | 0       | 0               | 100%         |
| CC6.6 - Security Incidents       | 5              | 5           | 0       | 0               | 100%         |
| CC6.7 - Threat Management        | 4              | 3           | 1       | 0               | 75%          |
| CC6.8 - Data Loss Prevention     | 5              | 4           | 1       | 0               | 80%          |
| CC7.1 - Vulnerability Management | 6              | 5           | 1       | 0               | 83%          |
| CC7.2 - Anomaly Detection        | 4              | 4           | 0       | 0               | 100%         |
| CC8.1 - Change Management        | 7              | 7           | 0       | 0               | 100%         |
| A1.1 - Availability              | 8              | 7           | 1       | 0               | 88%          |
| A1.2 - Backup & Recovery         | 6              | 5           | 1       | 0               | 83%          |
| A1.3 - Disaster Recovery         | 4              | 3           | 0       | 1               | 75%          |

---

## Control Details

### CC6.1 - Logical and Physical Access

| Control ID | Control Description                         | Implementation Status | Evidence                                        |
| ---------- | ------------------------------------------- | --------------------- | ----------------------------------------------- |
| CC6.1.1    | Access controls implemented for all systems | **Implemented**       | IAM configuration, access control matrices      |
| CC6.1.2    | Multi-factor authentication required        | **Implemented**       | MFA policy enforcement, FIDO2/TOTP support      |
| CC6.1.3    | Password policy enforced                    | **Implemented**       | Password policy configuration, enforcement logs |
| CC6.1.4    | Session timeout controls                    | **Implemented**       | 15-minute idle timeout, 8-hour absolute timeout |
| CC6.1.5    | Account lockout after failed attempts       | **Implemented**       | 5 attempts, 30-minute lockout                   |
| CC6.1.6    | Privileged access management                | **Implemented**       | PAM system, JIT access, session recording       |
| CC6.1.7    | Service account management                  | **Implemented**       | Service account inventory, key rotation         |
| CC6.1.8    | API authentication                          | **Implemented**       | API keys, mTLS, rate limiting                   |

### CC6.2 - Access Removal

| Control ID | Control Description             | Implementation Status | Evidence                                        |
| ---------- | ------------------------------- | --------------------- | ----------------------------------------------- |
| CC6.2.1    | Access removal upon termination | **Implemented**       | Offboarding checklist, automated deprovisioning |
| CC6.2.2    | Access removal upon role change | **Implemented**       | Role change workflow, access recertification    |
| CC6.2.3    | Periodic access reviews         | **Implemented**       | Quarterly access reviews, documented process    |
| CC6.2.4    | Orphaned account detection      | **Implemented**       | Weekly scans, automated alerts                  |

### CC6.3 - Access Authorization

| Control ID | Control Description            | Implementation Status | Evidence                               |
| ---------- | ------------------------------ | --------------------- | -------------------------------------- |
| CC6.3.1    | Formal access request process  | **Implemented**       | Ticketing system, approval workflow    |
| CC6.3.2    | Manager approval required      | **Implemented**       | Approval workflow configuration        |
| CC6.3.3    | Least privilege principle      | **Implemented**       | Role-based access, minimal permissions |
| CC6.3.4    | Access provisioning automation | **Implemented**       | IAM automation, SCIM provisioning      |
| CC6.3.5    | Access documentation           | **Implemented**       | Access registry, audit trail           |
| CC6.3.6    | Third-party access management  | **Implemented**       | Vendor access policy, NDA requirements |

### CC6.6 - Security Incidents

| Control ID | Control Description          | Implementation Status | Evidence                              |
| ---------- | ---------------------------- | --------------------- | ------------------------------------- |
| CC6.6.1    | Incident response procedures | **Implemented**       | Incident Response Policy, runbooks    |
| CC6.6.2    | Incident classification      | **Implemented**       | SEV-1 through SEV-4 classification    |
| CC6.6.3    | Incident escalation          | **Implemented**       | Escalation matrix, on-call rotation   |
| CC6.6.4    | Incident documentation       | **Implemented**       | Incident tickets, post-mortem process |
| CC6.6.5    | Incident metrics tracking    | **Implemented**       | MTTD, MTTR metrics, trend analysis    |

### CC6.7 - Threat Management

| Control ID | Control Description         | Implementation Status | Evidence                                       |
| ---------- | --------------------------- | --------------------- | ---------------------------------------------- |
| CC6.7.1    | Threat intelligence program | **Implemented**       | Threat feed subscriptions, IOC integration     |
| CC6.7.2    | Vulnerability scanning      | **Implemented**       | Weekly scans, automated remediation            |
| CC6.7.3    | Penetration testing         | **Partial**           | Annual test scheduled, last test: 6 months ago |
| CC6.7.4    | Threat modeling             | **Implemented**       | STRIDE analysis, attack trees documented       |

**Gap Analysis - CC6.7.3:**

- **Current State:** Annual penetration test scheduled but not yet completed for current period
- **Remediation Plan:** Schedule penetration test within 30 days
- **Compensating Control:** Continuous vulnerability scanning, security monitoring

### CC6.8 - Data Loss Prevention

| Control ID | Control Description   | Implementation Status | Evidence                                                   |
| ---------- | --------------------- | --------------------- | ---------------------------------------------------------- |
| CC6.8.1    | Data classification   | **Implemented**       | Classification scheme, data inventory                      |
| CC6.8.2    | Encryption at rest    | **Implemented**       | AES-256-GCM, key management                                |
| CC6.8.3    | Encryption in transit | **Implemented**       | TLS 1.3, mTLS for internal                                 |
| CC6.8.4    | DLP controls          | **Partial**           | PII detection implemented, exfiltration monitoring partial |
| CC6.8.5    | Data retention        | **Implemented**       | Retention policies, automated deletion                     |

**Gap Analysis - CC6.8.4:**

- **Current State:** PII detection active, network egress monitoring not fully deployed
- **Remediation Plan:** Deploy network DLP sensors within 60 days
- **Compensating Control:** Access logging, anomaly detection

### CC7.1 - Vulnerability Management

| Control ID | Control Description             | Implementation Status | Evidence                             |
| ---------- | ------------------------------- | --------------------- | ------------------------------------ |
| CC7.1.1    | Vulnerability scanning schedule | **Implemented**       | Weekly automated scans               |
| CC7.1.2    | Vulnerability remediation SLAs  | **Implemented**       | Critical: 24h, High: 7d, Medium: 30d |
| CC7.1.3    | Patch management                | **Implemented**       | Automated patching, test environment |
| CC7.1.4    | Dependency scanning             | **Implemented**       | SCA tools, automated PRs             |
| CC7.1.5    | Security testing in CI/CD       | **Partial**           | SAST implemented, DAST in progress   |
| CC7.1.6    | Vulnerability metrics           | **Implemented**       | Dashboard, trend tracking            |

**Gap Analysis - CC7.1.5:**

- **Current State:** SAST fully integrated, DAST requires additional configuration
- **Remediation Plan:** Complete DAST integration within 45 days
- **Compensating Control:** Manual penetration testing, security review process

### CC7.2 - Anomaly Detection

| Control ID | Control Description       | Implementation Status | Evidence                               |
| ---------- | ------------------------- | --------------------- | -------------------------------------- |
| CC7.2.1    | Security monitoring       | **Implemented**       | SIEM deployment, correlation rules     |
| CC7.2.2    | Anomaly detection         | **Implemented**       | ML-based detection, baseline alerting  |
| CC7.2.3    | Alert response procedures | **Implemented**       | Playbooks, on-call rotation            |
| CC7.2.4    | Log retention             | **Implemented**       | 1-year retention, tamper-proof storage |

### CC8.1 - Change Management

| Control ID | Control Description      | Implementation Status | Evidence                                  |
| ---------- | ------------------------ | --------------------- | ----------------------------------------- |
| CC8.1.1    | Change request process   | **Implemented**       | Ticketing system, approval workflow       |
| CC8.1.2    | Change risk assessment   | **Implemented**       | Risk matrix, impact analysis              |
| CC8.1.3    | Change approval process  | **Implemented**       | CAB process, documented approvals         |
| CC8.1.4    | Change testing           | **Implemented**       | Test environment, automated testing       |
| CC8.1.5    | Change documentation     | **Implemented**       | Change logs, release notes                |
| CC8.1.6    | Change rollback          | **Implemented**       | Rollback procedures, automated rollback   |
| CC8.1.7    | Emergency change process | **Implemented**       | Emergency CAB, post-implementation review |

### A1.1 - Availability

| Control ID | Control Description    | Implementation Status | Evidence                                         |
| ---------- | ---------------------- | --------------------- | ------------------------------------------------ |
| A1.1.1     | Capacity planning      | **Implemented**       | Capacity dashboard, forecasting                  |
| A1.1.2     | Performance monitoring | **Implemented**       | Real-time monitoring, alerting                   |
| A1.1.3     | SLA definitions        | **Implemented**       | 99.9% uptime SLA, SLOs defined                   |
| A1.1.4     | Incident management    | **Implemented**       | Incident process, escalation                     |
| A1.1.5     | Service continuity     | **Partial**           | DR site operational, failover testing incomplete |
| A1.1.6     | Maintenance windows    | **Implemented**       | Scheduled windows, customer notification         |
| A1.1.7     | Health checks          | **Implemented**       | Liveness/readiness probes, circuit breakers      |
| A1.1.8     | Load balancing         | **Implemented**       | Multi-zone deployment, auto-scaling              |

**Gap Analysis - A1.1.5:**

- **Current State:** DR site deployed, failover testing not completed quarterly
- **Remediation Plan:** Schedule quarterly DR tests, document results
- **Compensating Control:** Multi-zone redundancy, automated failover

### A1.2 - Backup and Recovery

| Control ID | Control Description   | Implementation Status | Evidence                                            |
| ---------- | --------------------- | --------------------- | --------------------------------------------------- |
| A1.2.1     | Backup policy         | **Implemented**       | Daily backups, 30-day retention                     |
| A1.2.2     | Backup encryption     | **Implemented**       | AES-256 encryption, separate key management         |
| A1.2.3     | Backup testing        | **Partial**           | Monthly restore tests, need quarterly full recovery |
| A1.2.4     | Backup monitoring     | **Implemented**       | Backup success alerts, failure escalation           |
| A1.2.5     | Geographic redundancy | **Implemented**       | Cross-region backup replication                     |
| A1.2.6     | Recovery procedures   | **Implemented**       | Documented runbooks, tested procedures              |

**Gap Analysis - A1.2.3:**

- **Current State:** Monthly component restore tests, full system recovery not tested quarterly
- **Remediation Plan:** Schedule quarterly full recovery tests
- **Compensating Control:** Component-level recovery validated, documented procedures

### A1.3 - Disaster Recovery

| Control ID | Control Description   | Implementation Status | Evidence                                |
| ---------- | --------------------- | --------------------- | --------------------------------------- |
| A1.3.1     | DR plan documentation | **Implemented**       | DR runbook, contact lists               |
| A1.3.2     | DR site readiness     | **Implemented**       | Hot standby, data replication           |
| A1.3.3     | DR testing            | **Partial**           | Annual test completed, need semi-annual |
| A1.3.4     | RTO/RPO definitions   | **Not Implemented**   | RTO/RPO not formally defined            |

**Gap Analysis - A1.3.3:**

- **Current State:** Annual DR test completed
- **Remediation Plan:** Schedule semi-annual DR tests
- **Compensating Control:** Continuous replication, automated failover capability

**Gap Analysis - A1.3.4:**

- **Current State:** RTO/RPO targets discussed but not formally documented
- **Remediation Plan:** Define and document RTO (4 hours) and RPO (1 hour)
- **Compensating Control:** Current architecture supports RTO < 4 hours, RPO < 1 hour

---

## Remediation Roadmap

### High Priority (30 Days)

| Control | Gap                      | Remediation                       | Owner      | Due Date   |
| ------- | ------------------------ | --------------------------------- | ---------- | ---------- |
| CC6.7.3 | Penetration test overdue | Schedule and complete annual test | Security   | 2026-03-20 |
| A1.3.4  | RTO/RPO undefined        | Document formal RTO/RPO targets   | Operations | 2026-03-01 |

### Medium Priority (60 Days)

| Control | Gap                   | Remediation                      | Owner      | Due Date   |
| ------- | --------------------- | -------------------------------- | ---------- | ---------- |
| CC6.8.4 | DLP incomplete        | Deploy network DLP sensors       | Security   | 2026-04-19 |
| A1.1.5  | DR testing incomplete | Complete quarterly failover test | Operations | 2026-04-19 |

### Low Priority (90 Days)

| Control | Gap                   | Remediation                           | Owner      | Due Date   |
| ------- | --------------------- | ------------------------------------- | ---------- | ---------- |
| CC7.1.5 | DAST incomplete       | Complete DAST integration             | DevOps     | 2026-05-19 |
| A1.2.3  | Full recovery testing | Complete quarterly full recovery test | Operations | 2026-05-19 |
| A1.3.3  | DR test frequency     | Implement semi-annual DR testing      | Operations | 2026-05-19 |

---

## Control Testing Evidence

### Automated Evidence Collection

| Evidence Type       | Collection Method | Frequency  | Retention |
| ------------------- | ----------------- | ---------- | --------- |
| Access Logs         | SIEM aggregation  | Real-time  | 1 year    |
| Change Records      | Ticketing system  | Continuous | 3 years   |
| Vulnerability Scans | Security tooling  | Weekly     | 2 years   |
| Backup Logs         | Backup system     | Daily      | 1 year    |
| Incident Records    | Incident system   | Continuous | 3 years   |

### Manual Evidence Collection

| Evidence Type     | Collection Method      | Frequency   | Responsible   |
| ----------------- | ---------------------- | ----------- | ------------- |
| Access Reviews    | Review meetings        | Quarterly   | Security Team |
| Policy Reviews    | Document review        | Annual      | Compliance    |
| Penetration Tests | Third-party assessment | Annual      | Security Team |
| DR Tests          | Full failover exercise | Semi-annual | Operations    |

---

## Compliance Monitoring

### Key Metrics

| Metric                       | Target | Current | Trend     |
| ---------------------------- | ------ | ------- | --------- |
| Control Compliance           | 95%    | 91%     | Improving |
| Vulnerability SLA Compliance | 100%   | 98%     | Stable    |
| Access Review Completion     | 100%   | 100%    | Stable    |
| Incident Response SLA        | 100%   | 100%    | Stable    |
| Backup Success Rate          | 100%   | 99.9%   | Stable    |

### Continuous Monitoring

- Real-time compliance dashboard
- Automated control testing
- Exception alerting
- Quarterly compliance reviews

---

## Document Control

| Version | Date       | Author          | Changes            |
| ------- | ---------- | --------------- | ------------------ |
| 1.0.0   | 2026-02-18 | Compliance Team | Initial assessment |

---

## Approval

| Role            | Name               | Signature          | Date       |
| --------------- | ------------------ | ------------------ | ---------- |
| CISO            | ********\_******** | ********\_******** | ****\_**** |
| Compliance Lead | ********\_******** | ********\_******** | ****\_**** |

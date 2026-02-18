# Access Review Template

**Document Type:** Compliance Template  
**Version:** 1.0.0  
**Review Frequency:** Quarterly  
**Owner:** Security Team

---

## Purpose

This template provides a standardized format for conducting quarterly access reviews as required by SOC 2 compliance controls CC6.2.3 and CC6.3.5.

---

## Review Information

| Field             | Value                                                     |
| ----------------- | --------------------------------------------------------- |
| **Review Period** | Q\_ 20** (Start Date: \_** to End Date: \_\_\_)           |
| **Review Type**   | [ ] User Access [ ] Privileged Access [ ] Service Account |
| **Reviewer Name** | ************\_************                                |
| **Reviewer Role** | ************\_************                                |
| **Review Date**   | ************\_************                                |

---

## Section 1: User Access Review

### 1.1 New Access Grants

List all new access grants during the review period:

| User ID | User Name | Access Granted | Date Granted | Approver | Business Justification | Appropriate?   |
| ------- | --------- | -------------- | ------------ | -------- | ---------------------- | -------------- |
|         |           |                |              |          |                        | [ ] Yes [ ] No |
|         |           |                |              |          |                        | [ ] Yes [ ] No |
|         |           |                |              |          |                        | [ ] Yes [ ] No |
|         |           |                |              |          |                        | [ ] Yes [ ] No |
|         |           |                |              |          |                        | [ ] Yes [ ] No |

**Findings:**

- Inappropriate access grants: \_\_\_
- Missing justification: \_\_\_
- Actions required: \_\_\_

### 1.2 Access Removals

List all access removals during the review period:

| User ID | User Name | Access Removed | Date Removed | Reason | Timely Removal? |
| ------- | --------- | -------------- | ------------ | ------ | --------------- |
|         |           |                |              |        | [ ] Yes [ ] No  |
|         |           |                |              |        | [ ] Yes [ ] No  |
|         |           |                |              |        | [ ] Yes [ ] No  |
|         |           |                |              |        | [ ] Yes [ ] No  |

**Findings:**

- Delayed removals: \_\_\_
- Average removal time: \_\_\_ days
- Actions required: \_\_\_

### 1.3 Current Access Verification

Verify current access is appropriate:

| User ID | User Name | Role | Current Access | Last Activity | Access Appropriate? | Action |
| ------- | --------- | ---- | -------------- | ------------- | ------------------- | ------ |
|         |           |      |                |               | [ ] Yes [ ] No      |        |
|         |           |      |                |               | [ ] Yes [ ] No      |        |
|         |           |      |                |               | [ ] Yes [ ] No      |        |
|         |           |      |                |               | [ ] Yes [ ] No      |        |
|         |           |      |                |               | [ ] Yes [ ] No      |        |

**Summary:**

- Total users reviewed: \_\_\_
- Access appropriate: \_\_\_
- Access requiring modification: \_\_\_
- Access requiring removal: \_\_\_

---

## Section 2: Privileged Access Review

### 2.1 Privileged Account Inventory

| Account ID | Account Name | Privilege Level                            | Owner | Last Used | Still Required? |
| ---------- | ------------ | ------------------------------------------ | ----- | --------- | --------------- |
|            |              | [ ] Admin [ ] DBA [ ] Security [ ] Network |       |           | [ ] Yes [ ] No  |
|            |              | [ ] Admin [ ] DBA [ ] Security [ ] Network |       |           | [ ] Yes [ ] No  |
|            |              | [ ] Admin [ ] DBA [ ] Security [ ] Network |       |           | [ ] Yes [ ] No  |

### 2.2 Privileged Access Activity

| Account ID | Actions Performed | Date Range | Anomalous Activity? |
| ---------- | ----------------- | ---------- | ------------------- |
|            |                   |            | [ ] Yes [ ] No      |
|            |                   |            | [ ] Yes [ ] No      |
|            |                   |            | [ ] Yes [ ] No      |

**Findings:**

- Dormant privileged accounts: \_\_\_
- Anomalous activity detected: \_\_\_
- Actions required: \_\_\_

### 2.3 Just-In-Time Access Review

| Request ID | Requester | Privilege Requested | Duration | Justification | Approved By | Appropriate?   |
| ---------- | --------- | ------------------- | -------- | ------------- | ----------- | -------------- |
|            |           |                     |          |               |             | [ ] Yes [ ] No |
|            |           |                     |          |               |             | [ ] Yes [ ] No |
|            |           |                     |          |               |             | [ ] Yes [ ] No |

---

## Section 3: Service Account Review

### 3.1 Service Account Inventory

| Account ID | Account Name | System/Service | Owner | Key Rotation Date | Key Age Appropriate?      |
| ---------- | ------------ | -------------- | ----- | ----------------- | ------------------------- |
|            |              |                |       |                   | [ ] Yes [ ] No (>90 days) |
|            |              |                |       |                   | [ ] Yes [ ] No (>90 days) |
|            |              |                |       |                   | [ ] Yes [ ] No (>90 days) |

**Findings:**

- Service accounts with overdue key rotation: \_\_\_
- Orphaned service accounts: \_\_\_
- Actions required: \_\_\_

### 3.2 API Key Review

| Key ID | Key Name | Service | Owner | Last Used | Expiration | Status                             |
| ------ | -------- | ------- | ----- | --------- | ---------- | ---------------------------------- |
|        |          |         |       |           |            | [ ] Active [ ] Expired [ ] Revoked |
|        |          |         |       |           |            | [ ] Active [ ] Expired [ ] Revoked |
|        |          |         |       |           |            | [ ] Active [ ] Expired [ ] Revoked |

---

## Section 4: Role-Based Access Control Review

### 4.1 Role Definitions

| Role Name | Description | Permissions | Last Review | Changes Needed? |
| --------- | ----------- | ----------- | ----------- | --------------- |
|           |             |             |             | [ ] Yes [ ] No  |
|           |             |             |             | [ ] Yes [ ] No  |
|           |             |             |             | [ ] Yes [ ] No  |

### 4.2 Role Assignments

| Role Name | Users Assigned | Appropriate Assignments | Over-privileged Users |
| --------- | -------------- | ----------------------- | --------------------- |
|           |                |                         |                       |
|           |                |                         |                       |
|           |                |                         |                       |

---

## Section 5: Third-Party Access Review

### 5.1 Vendor Access

| Vendor Name | Access Type | Users | NDA on File    | Access Still Required? | Review Date |
| ----------- | ----------- | ----- | -------------- | ---------------------- | ----------- |
|             |             |       | [ ] Yes [ ] No | [ ] Yes [ ] No         |             |
|             |             |       | [ ] Yes [ ] No | [ ] Yes [ ] No         |             |
|             |             |       | [ ] Yes [ ] No | [ ] Yes [ ] No         |             |

**Findings:**

- Vendors with expired NDAs: \_\_\_
- Vendors requiring access removal: \_\_\_
- Actions required: \_\_\_

---

## Section 6: Orphaned Account Detection

### 6.1 Inactive Accounts

Accounts with no activity for 90+ days:

| Account ID | Account Name | Last Activity | Days Inactive | Action                               |
| ---------- | ------------ | ------------- | ------------- | ------------------------------------ |
|            |              |               |               | [ ] Disable [ ] Investigate [ ] Keep |
|            |              |               |               | [ ] Disable [ ] Investigate [ ] Keep |
|            |              |               |               | [ ] Disable [ ] Investigate [ ] Keep |

### 6.2 Orphaned Accounts

Accounts with no assigned owner:

| Account ID | Account Name | Type                             | Created Date | Action                       |
| ---------- | ------------ | -------------------------------- | ------------ | ---------------------------- |
|            |              | [ ] User [ ] Service [ ] Generic |              | [ ] Disable [ ] Assign Owner |
|            |              | [ ] User [ ] Service [ ] Generic |              | [ ] Disable [ ] Assign Owner |

---

## Section 7: Access Anomalies

### 7.1 Unusual Access Patterns

| User ID | Anomaly Type                                                 | Description | Date | Investigated?  | Resolution |
| ------- | ------------------------------------------------------------ | ----------- | ---- | -------------- | ---------- |
|         | [ ] Off-hours [ ] Unusual location [ ] Bulk access [ ] Other |             |      | [ ] Yes [ ] No |            |
|         | [ ] Off-hours [ ] Unusual location [ ] Bulk access [ ] Other |             |      | [ ] Yes [ ] No |            |

### 7.2 Failed Access Attempts

| User ID | Failed Attempts | Time Period | Account Locked? | Investigated?  |
| ------- | --------------- | ----------- | --------------- | -------------- |
|         |                 |             | [ ] Yes [ ] No  | [ ] Yes [ ] No |
|         |                 |             | [ ] Yes [ ] No  | [ ] Yes [ ] No |

---

## Section 8: Compliance Metrics

### 8.1 Key Performance Indicators

| Metric                           | Target    | Actual       | Status              |
| -------------------------------- | --------- | ------------ | ------------------- |
| Access review completion rate    | 100%      | \_\_\_%      | [ ] Met [ ] Not Met |
| Inappropriate access remediation | <5%       | \_\_\_%      | [ ] Met [ ] Not Met |
| Access removal timeliness        | <24 hours | \_\_\_ hours | [ ] Met [ ] Not Met |
| Privileged account review        | 100%      | \_\_\_%      | [ ] Met [ ] Not Met |
| Service account key rotation     | 100%      | \_\_\_%      | [ ] Met [ ] Not Met |
| Orphaned account remediation     | 0         | \_\_\_       | [ ] Met [ ] Not Met |

### 8.2 Trend Analysis

| Metric              | Previous Quarter | Current Quarter | Trend                      |
| ------------------- | ---------------- | --------------- | -------------------------- |
| Total users         |                  |                 | [ ] Up [ ] Down [ ] Stable |
| Privileged accounts |                  |                 | [ ] Up [ ] Down [ ] Stable |
| Service accounts    |                  |                 | [ ] Up [ ] Down [ ] Stable |
| Access anomalies    |                  |                 | [ ] Up [ ] Down [ ] Stable |
| Orphaned accounts   |                  |                 | [ ] Up [ ] Down [ ] Stable |

---

## Section 9: Remediation Actions

### 9.1 Required Actions

| Action ID | Description | Priority                    | Owner | Due Date | Status                                |
| --------- | ----------- | --------------------------- | ----- | -------- | ------------------------------------- |
|           |             | [ ] High [ ] Medium [ ] Low |       |          | [ ] Open [ ] In Progress [ ] Complete |
|           |             | [ ] High [ ] Medium [ ] Low |       |          | [ ] Open [ ] In Progress [ ] Complete |
|           |             | [ ] High [ ] Medium [ ] Low |       |          | [ ] Open [ ] In Progress [ ] Complete |

### 9.2 Follow-up from Previous Review

| Action ID | Previous Finding | Remediation Status                           | Evidence |
| --------- | ---------------- | -------------------------------------------- | -------- |
|           |                  | [ ] Complete [ ] In Progress [ ] Not Started |          |
|           |                  | [ ] Complete [ ] In Progress [ ] Not Started |          |

---

## Section 10: Certification

### 10.1 Reviewer Certification

I certify that I have reviewed the access controls listed in this document and that:

- [ ] All user access has been reviewed and is appropriate for business needs
- [ ] All privileged access has been reviewed and is still required
- [ ] All service accounts have been inventoried and keys rotated as required
- [ ] All orphaned accounts have been identified and remediated
- [ ] All anomalies have been investigated and resolved
- [ ] All remediation actions have been assigned and tracked

**Reviewer Signature:** ************\_************  
**Date:** ************\_************

### 10.2 Manager Approval

I have reviewed the findings and remediation actions documented in this access review and approve the certification.

**Manager Name:** ************\_************  
**Manager Signature:** ************\_************  
**Date:** ************\_************

---

## Appendix A: Review Checklist

### Pre-Review Preparation

- [ ] Obtain current user access report from IAM system
- [ ] Obtain privileged account inventory
- [ ] Obtain service account inventory
- [ ] Obtain access logs for review period
- [ ] Review previous quarter's remediation actions
- [ ] Schedule review meetings with system owners

### During Review

- [ ] Verify each user's access is appropriate for their role
- [ ] Confirm privileged access is still required
- [ ] Validate service account key rotation
- [ ] Identify orphaned and inactive accounts
- [ ] Investigate access anomalies
- [ ] Document all findings

### Post-Review

- [ ] Complete remediation action assignments
- [ ] Submit review for manager approval
- [ ] Archive review documentation
- [ ] Update access review tracker
- [ ] Schedule next quarterly review

---

## Appendix B: Evidence Requirements

| Evidence Type        | Description                            | Retention Period |
| -------------------- | -------------------------------------- | ---------------- |
| Access Reports       | Screenshots or exports from IAM system | 3 years          |
| Review Workpapers    | This completed template                | 3 years          |
| Remediation Evidence | Screenshots showing actions completed  | 3 years          |
| Approval Records     | Signed certifications                  | 3 years          |

---

## Document Control

| Version | Date       | Author          | Changes          |
| ------- | ---------- | --------------- | ---------------- |
| 1.0.0   | 2026-02-18 | Compliance Team | Initial template |

---

## Distribution

| Role          | Distribution Method |
| ------------- | ------------------- |
| Security Team | Document repository |
| IT Management | Email notification  |
| Compliance    | Document repository |
| Auditors      | Upon request        |

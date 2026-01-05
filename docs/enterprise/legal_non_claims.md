# Path and File Name : /home/ransomeye/rebuild/docs/enterprise/legal_non_claims.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Legal Non-Claims Declaration - Explicit statement of what RansomEye does NOT claim (PROMPT-65-D)

# Legal Non-Claims Declaration (PROMPT-65-D)

**Version:** v1.0.0-enterprise-ship  
**Date:** 2026-01-28  
**Status:** ✅ **IMPLEMENTED**

---

## Purpose

This document explicitly states **what RansomEye does NOT claim**, **what it intentionally does NOT do**, and **where responsibility boundaries lie**. This prevents legal overreach and establishes clear expectations.

---

## What RansomEye Does NOT Claim

### 1. Perfect Security

**RansomEye does NOT claim:**
- 100% prevention of all security incidents
- Zero false positives or false negatives
- Protection against all attack vectors
- Immunity to zero-day vulnerabilities
- Perfect detection accuracy

**What RansomEye DOES claim:**
- Provable detection of post-ship modifications
- Fail-closed enforcement on violations
- Immutable audit trail
- Vendor non-repudiation

### 2. Absolute Immutability

**RansomEye does NOT claim:**
- Physical immutability of hardware
- Protection against kernel-level attacks
- Protection against firmware attacks
- Protection against hardware backdoors
- Protection against physical tampering

**What RansomEye DOES claim:**
- Cryptographic immutability of software artifacts
- Detection of software modifications within ≤5 minutes
- Immutable audit trail of all changes
- Fail-closed enforcement on violations

### 3. Vendor Powerlessness

**RansomEye does NOT claim:**
- Vendor cannot access customer data (with proper credentials)
- Vendor cannot provide support or maintenance
- Vendor cannot update system (with proper procedures)
- Vendor cannot respond to security incidents

**What RansomEye DOES claim:**
- Vendor cannot override ship seal enforcement
- Vendor cannot bypass verifier checks
- Vendor cannot disable assurance mechanisms
- Vendor cannot modify system without detection

### 4. Legal Compliance

**RansomEye does NOT claim:**
- Automatic compliance with all regulations
- Legal advice or interpretation
- Regulatory approval or certification
- Legal defense or indemnification

**What RansomEye DOES claim:**
- Evidence artifacts suitable for regulatory review
- Audit trail suitable for compliance verification
- Documentation suitable for legal proceedings
- Technical controls aligned with security best practices

### 5. Performance Guarantees

**RansomEye does NOT claim:**
- Specific performance metrics
- Uptime guarantees
- Response time guarantees
- Throughput guarantees

**What RansomEye DOES claim:**
- Detection time ≤5 minutes for modifications
- Fail-closed enforcement on violations
- Immutable audit trail
- Vendor non-repudiation

---

## What RansomEye Intentionally Does NOT Do

### 1. Silent Failures

**RansomEye intentionally does NOT:**
- Fail silently without logging
- Continue operation after violations
- Suppress violation alerts
- Hide integrity failures

**RansomEye intentionally DOES:**
- Fail-closed on all violations
- Log all violations to immutable audit trail
- Generate SYSTEM_INTEGRITY_VIOLATION entries
- Block operation on violations

### 2. Vendor Override Mechanisms

**RansomEye intentionally does NOT:**
- Provide backdoor access for vendors
- Allow vendor override of protections
- Include hidden disable flags
- Support secret recovery mechanisms

**RansomEye intentionally DOES:**
- Enforce ship seal without vendor override
- Require vendor non-repudiation verification
- Block all bypass attempts
- Detect all override attempts

### 3. Customer Lock-In

**RansomEye intentionally does NOT:**
- Prevent customer verification
- Require vendor assistance for verification
- Hide verification mechanisms
- Restrict evidence access

**RansomEye intentionally DOES:**
- Provide independent customer verification
- Enable offline verification
- Provide evidence bundle for regulators
- Support third-party audits

### 4. Legal Interpretation

**RansomEye intentionally does NOT:**
- Provide legal advice
- Interpret regulations
- Guarantee compliance
- Provide legal defense

**RansomEye intentionally DOES:**
- Provide technical evidence
- Support regulatory review
- Enable legal proceedings
- Document technical controls

---

## Responsibility Boundaries

### Vendor Responsibilities

**RansomEye vendor is responsible for:**
- Providing software that meets specifications
- Maintaining ship seal integrity
- Providing evidence artifacts
- Supporting customer verification

**RansomEye vendor is NOT responsible for:**
- Customer's security policies
- Customer's compliance decisions
- Customer's legal interpretation
- Customer's incident response

### Customer Responsibilities

**Customer is responsible for:**
- Implementing security policies
- Making compliance decisions
- Interpreting regulations
- Responding to security incidents

**Customer is NOT responsible for:**
- Vendor's software development
- Vendor's ship seal maintenance
- Vendor's evidence generation
- Vendor's verification support

### Regulator Responsibilities

**Regulator is responsible for:**
- Reviewing evidence artifacts
- Verifying technical controls
- Interpreting regulations
- Making compliance determinations

**Regulator is NOT responsible for:**
- Vendor's software development
- Customer's security policies
- Legal interpretation
- Incident response

---

## Limitations and Exclusions

### Technical Limitations

**RansomEye has the following technical limitations:**
- Cannot detect kernel-level file system manipulation
- Cannot detect in-memory binary modification
- Cannot protect against hardware backdoors
- Cannot protect against firmware attacks
- Small timing window between check and execution

### Operational Limitations

**RansomEye has the following operational limitations:**
- Requires proper system configuration
- Requires database access for full audit chain
- Requires systemd for continuous verification
- Requires file system integrity

### Legal Limitations

**RansomEye has the following legal limitations:**
- Does not provide legal advice
- Does not guarantee regulatory compliance
- Does not provide legal defense
- Does not indemnify customers

---

## Disclaimers

### Security Disclaimer

**RansomEye provides security controls but:**
- Cannot guarantee 100% security
- Cannot prevent all attacks
- Cannot eliminate all risks
- Cannot replace security best practices

### Compliance Disclaimer

**RansomEye provides evidence artifacts but:**
- Does not guarantee regulatory compliance
- Does not provide legal interpretation
- Does not replace legal counsel
- Does not guarantee regulatory approval

### Performance Disclaimer

**RansomEye provides technical controls but:**
- Does not guarantee specific performance
- Does not guarantee uptime
- Does not guarantee response times
- Does not guarantee throughput

---

## Warranty Limitations

### No Warranty

**RansomEye is provided "AS IS" without warranty of any kind, express or implied, including but not limited to:**
- Warranties of merchantability
- Warranties of fitness for a particular purpose
- Warranties of non-infringement
- Warranties of security or compliance

### Limitation of Liability

**RansomEye vendor's liability is limited to:**
- Direct damages only
- Maximum liability not exceeding purchase price
- No liability for indirect, incidental, or consequential damages
- No liability for lost profits or data

---

## Indemnification

### Customer Indemnification

**Customer agrees to indemnify RansomEye vendor against:**
- Customer's use of RansomEye
- Customer's security policies
- Customer's compliance decisions
- Customer's legal interpretation

### Vendor Indemnification

**RansomEye vendor does NOT indemnify customer against:**
- Security incidents
- Compliance failures
- Legal proceedings
- Regulatory actions

---

## Governing Law

### Applicable Law

**This declaration is governed by:**
- Applicable local laws
- Applicable regulations
- Applicable industry standards
- Applicable contractual terms

### Dispute Resolution

**Disputes shall be resolved through:**
- Good faith negotiation
- Mediation (if required)
- Arbitration (if required)
- Legal proceedings (if required)

---

## Updates and Modifications

### Declaration Updates

**This declaration may be updated:**
- With new versions of RansomEye
- To reflect technical changes
- To address regulatory requirements
- To clarify responsibilities

### Notification

**Customers will be notified of:**
- Material changes to this declaration
- Updates to responsibility boundaries
- Changes to limitations or exclusions
- Updates to disclaimers

---

## Acceptance

### Customer Acceptance

**By using RansomEye, customer acknowledges:**
- Understanding of non-claims
- Acceptance of responsibility boundaries
- Awareness of limitations
- Agreement to disclaimers

### Vendor Acknowledgment

**RansomEye vendor acknowledges:**
- Customer's right to independent verification
- Customer's right to evidence artifacts
- Customer's right to regulatory review
- Customer's right to legal proceedings

---

## Conclusion

This Legal Non-Claims Declaration establishes **clear boundaries** for RansomEye v1.0.0-enterprise-ship:

- **What RansomEye claims:** Technical controls and evidence artifacts
- **What RansomEye does NOT claim:** Perfect security, absolute immutability, legal compliance
- **What RansomEye intentionally does NOT do:** Silent failures, vendor overrides, customer lock-in
- **Where responsibility lies:** Vendor (software), Customer (policies), Regulator (review)

This declaration prevents legal overreach and establishes clear expectations for all parties.

---

© RansomEye.Tech | Support: Gagan@RansomEye.Tech


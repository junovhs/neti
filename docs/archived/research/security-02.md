# SOTA Report: Hardcoded Secrets Detection - Low FP (Security-02)

**Date**: January 2026  
**Subject**: Advanced techniques for identifying credentials, entropy-based heuristics, and the move toward active verification.

## 1. Executive Summary
Traditional secret scanning relies on Regular Expressions (Regex) and Shannon Entropy, leading to high False Positive (FP) rates (often >50%). State-of-the-Art (SOTA) research has pivoted toward **Active Verification** (real-time API validation) and **Hybrid AI Classification** (semantic context analysis) to reduce alert fatigue and increase precision.

---

## 2. The Entropy Problem
Shannon Entropy measures the randomness of a string. While secrets are random, so are UUIDs, Hashes (MD5/SHA), and compressed data.

*   **SOTA Pitfall**: "Blind Entropy." Flagging all high-entropy strings regardless of context.
*   **SOTA Fix**: **Entropy Windows**. Only analyzing entropy within certain file types (.env, .json, .config) or near specific variable assignments (e.g., `PASS`, `KEY`, `TOKEN`).

---

## 3. Tooling SOTA: TruffleHog and DeepSource

### 3.1 TruffleHog v3 (Active Verification)
The current SOTA for open-source verification.

*   **Mechanism**: **Credential Detectors + API Probing**.
    1.  TruffleHog identifies a potential AWS key using a pattern.
    2.  It immediately sends a "Dry Run" request to the AWS STS API to check if the key is valid.
*   **SOTA Result**: It filters out "Fake" or "Inactive" secrets, ensuring that analysts only spend time on live leaks.

### 3.2 DeepSource Narada (Hybrid AI Engine)
Moves beyond patterns to "Semantic Intent."

*   **Mechanism**: **Narada Model**. A machine learning model trained on millions of examples of real vs. fake secrets.
*   **SOTA Advantage**: It recognizes that a high-entropy string inside a `test/` directory named `dummy_key` is a false positive, even if it matches the pattern of a real key.
*   **Performance**: Claimed 93% reduction in FP compared to standard regex scanners.

---

## 4. Context-Aware Detection Hierarchies
SOTA scanners now follow a tiered validation process:
1.  **Regex Match**: Does it look like a secret?
2.  **Entropy Calculation**: Is it random enough?
3.  **Context Scoring**: Is it in a production file? Is it assigned to a credential-like variable?
4.  **Active Probe**: Is it live and valid?

---

## 5. Summary of SOTA Detection Capabilities

| Feature | Legacy Scanners | SOTA (2025/2026) |
| :--- | :--- | :--- |
| **Logic** | Regex / Keyword | Hybrid AI / Context-Aware |
| **Validation** | None | Active API Probing |
| **Scope** | Git History (Current) | DOCKER, S3, GCS, Logs |
| **Feedback Loop** | Bulk Report | Pre-commit / CI Gating |

---

## 6. References
1.  **TruffleHog**: [Truffle Security: The Gold Standard for Verification](https://trufflesecurity.com/trufflehog)
2.  **DeepSource Narada**: [AI-Powered Secrets Detection](https://deepsource.com/blog/announcing-secrets-analyzer-2-0)
3.  **Gitleaks SOTA**: [Fast and Lightweight Scan Patterns](https://gitleaks.io/)
4.  **Spectral**: [Check Point CloudGuard: Real-time Secret Blocking](https://spectralops.io/)
5.  **Rahman et al.**: (2022). *Anti-patterns in Secret Scanning (Why Developers Ignore Alerts)*.

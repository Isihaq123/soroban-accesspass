# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active  |

Only the latest released version receives security fixes.

---

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Disclosing a security bug publicly before a fix is available puts every protocol
that has deployed AccessPass at risk. Please follow this responsible disclosure
process instead.

### How to Report

Send a detailed report to:

**Email:** `security@soroban-accesspass.dev`

Encrypt your message with our PGP key if the details are sensitive (key available
on request via the email above).

Include as much of the following as possible:

- A clear description of the vulnerability
- The affected function(s) and file(s)
- Proof-of-concept code or a reproducible test case
- The potential impact (e.g., unauthorised role escalation, admin takeover)
- Any suggested mitigations

### What to Expect

| Timeline | Action |
|---|---|
| **Within 48 hours** | We acknowledge receipt of your report |
| **Within 7 days** | We assess severity and confirm whether it is a valid vulnerability |
| **Within 30 days** | We release a fix and a coordinated disclosure |
| **After release** | We publicly credit you (unless you prefer to remain anonymous) |

If we cannot reproduce the issue within 7 days we will ask follow-up questions.
If we disagree that the report is a vulnerability we will explain our reasoning.

---

## Scope

The following are **in scope**:

- Logic bugs in `contracts/accesspass/src/lib.rs` (role grant/revoke, delegation,
  admin transfer)
- Auth bypass vulnerabilities (missing `require_auth`, front-running vectors)
- State-rent issues that cause silent privilege loss or escalation
- Storage key collision attacks

The following are **out of scope**:

- Bugs in `soroban-sdk` itself (report those to the
  [Stellar Security Programme](https://stellar.org/security))
- Theoretical issues with no practical exploit path
- Issues that require the Admin key to already be compromised

---

## Severity Classification

We use the following labels when triaging reports:

| Severity | Criteria |
|---|---|
| **Critical** | Unauthorised Admin takeover, role escalation without valid credentials |
| **High** | Silent privilege loss (e.g., active role archived unexpectedly) |
| **Medium** | Incorrect event emission, storage waste with economic impact |
| **Low** | Edge-case behaviour that is technically incorrect but unexploitable |

---

## Acknowledgements

We are grateful to every researcher who takes the time to help keep this
project and its downstream protocols safe. Contributors who report valid
vulnerabilities will be listed here (with permission).

_No disclosures yet — this project is at v0.1.0._

---

## Related Resources

- [Stellar Bug Bounty Programme](https://stellar.org/security)
- [Soroban Security Best Practices](https://developers.stellar.org/docs/smart-contracts/security)
- [Responsible Disclosure — ISO/IEC 29147](https://www.iso.org/standard/72311.html)

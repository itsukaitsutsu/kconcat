# Security Policy — kconcat

## Scope and maintenance status

This policy covers this fork's code and packages, not an upstream service commitment. No supported-release matrix, guaranteed security-fix schedule, or response deadline has been established here. Report the exact version and commit; do not assume upstream's supported-version table or response promises apply to modified builds.

The [limited-support agreement](makethisforagreementuser/01-package-and-limited-support-agreement.md) is a draft. Purchased installation assistance does not automatically include ongoing code maintenance or bug fixes. That distinction does not waive mandatory security, personal-data protection, or consumer obligations, or an express commitment made with an order.

## Report vulnerabilities privately

**Do not open a public issue containing a vulnerability, exploit, credentials, or personal data.**

1. Check this fork's [Security page](https://github.com/itsukaitsutsu/kconcat/security). If private vulnerability reporting is enabled, use **Report a vulnerability** to contact this fork's maintainers privately.
2. If that option is unavailable, use a verified private security contact published by this fork's maintainers. No dedicated security email is specified in this policy. If none is published, ask for a private reporting channel without revealing vulnerability details; do not put the report in a public issue.
3. If the issue independently affects unmodified upstream Concat, use upstream's own current [security policy](https://github.com/jub0t/Concat/security/policy). Do not send fork-only defects or customer records to upstream by default.

Maintainers should enable private reporting or publish a working security contact before distributing customer packages. This document does not assert that private reporting is currently enabled.

Include the affected commit/build, operating system, component, impact, and minimal reproduction steps. Supply a small synthetic proof of concept where possible. Redact tokens, usernames, file paths containing personal details, payment/order data, and unrelated logs. Send private media/projects only when necessary and after agreeing on secure handling and retention.

## Relevant issues

Examples include:

- Malicious project, media, template, or effect files causing unexpected code execution or unauthorized file access/writes.
- Installer, dependency, model-download, or update tampering; missing or bypassed integrity checks.
- Remote/API authentication failures or unintended exposure of privileged operations.
- Unexpected media uploads, sensitive-data leakage, or insecure diagnostics.
- Account, payment, or delivery vulnerabilities in a provider-operated service, if one is deployed; report those to that service's verified private contact.

Do not test against other people's accounts, media, or infrastructure without authorization. Use controlled test data and disclose only what is necessary to explain the issue.

## Handling and disclosure

Maintainers should assess impact, coordinate remediation and disclosure with the reporter, and obtain consent before publishing identifying credit. No acknowledgement deadline, recurring update schedule, or next-release fix is promised by this policy. Avoid publishing exploitable details before maintainers have had an opportunity to assess and coordinate a response.

A personal-data incident is distinct from an ordinary bug. A provider operating customer services must investigate, mitigate, and make any notifications required by applicable law within the required timeframe. An absent maintenance commitment does not remove those obligations.

## Privacy and safer operation

The [Privacy Policy template](makethisforagreementuser/03-privacy-policy.md) requires verification and completion; it does not certify existing data flows or security controls. Verify actual application, installer, model-download, update, telemetry, and website behavior before making “fully offline” or “no data sent” claims.

Back up media and projects, obtain packages from a verified distributor, and check the release's source and integrity information. Treat remote/API access as privileged: verify authentication and exposure settings, prefer loopback access, and provide appropriate transport protection before exposing an interface beyond the local machine. These are precautions, not a certification that a given build implements them correctly.

# Calendar Management Security Re-Audit Report

Date: 2026-05-08  
Stage: 5 - Security Audit  
App: `apps/first-party/calendar-management`

## Executive Summary

The Stage 5 re-audit passes. The three prior security failures are fixed in the Rust planner and reflected in `plugin.json`: `Permission::Invite` no longer satisfies `Write`, `subscribe_calendar` accepts only sanitized host-managed subscription refs and rejects raw URL/provider fields recursively, and planner enum fields are validated before host-effect payload construction.

The app remains compliant with the Synapp sandbox model: no AI logic, no direct database/network/provider calls, no native-thread or native-only dependencies, and no credential storage paths were found in the reviewed app source. No security-gating blockers were identified.

## Re-Audit Scope

- `apps/first-party/calendar-management/src/lib.rs`
- `apps/first-party/calendar-management/plugin.json`
- `apps/first-party/calendar-management/synapp.app.json`
- `apps/first-party/calendar-management/Cargo.toml`
- `apps/first-party/calendar-management/README.md`
- `apps/first-party/calendar-management/ui/`
- `docs/calendar-management/architecture.md`
- `docs/calendar-management/contracts.md`
- `docs/calendar-management/plugin-schema-plan.md`
- `docs/calendar-management/ui-spec.md`
- `docs/calendar-management/ux-spec.md`

## Previous Findings

### Permission Escalation Fixed

Previous issue: `Permission::Invite` satisfied `Permission::Write`, allowing invite-only callers to pass write-gated operations.

Current status: fixed. `Invite` is additive only for `None`, `Read`, and `Invite`; it no longer satisfies `Write`, `Settings`, `Share`, `Respond`, or `Audit`. Regression coverage exists in the Rust test module.

### Subscription SSRF/Input Smuggling Fixed

Previous issue: `subscribe_calendar` accepted an arbitrary `HostObject` and could pass raw provider data into `SubscriptionSync`.

Current status: fixed. The handler now accepts only host-safe `subscription_ref` or `url_ref`, recursively rejects forbidden raw provider fields, and rejects string values containing `://`. `plugin.json` defines a strict `SubscriptionRequest` with `additionalProperties: false`.

### Planner Enum Validation Fixed

Previous issue: several schema-level enums were not enforced in Rust before constructing host-effect payloads.

Current status: fixed. Rust validates delete mode, event scopes, RSVP response/scope, notification scope, and reminder action before planning host effects. Unknown values return `InvalidInput`.

## Architecture Compliance

| Rule | Status | Notes |
| --- | --- | --- |
| Zero AI logic | PASS | No OpenAI, Anthropic, LangChain, LLM prompts, or AI routing logic found in app source. |
| Network/DB isolation | PASS | Rust uses only planner dependencies; UI host calls go through the Synapp bridge. No direct provider, TCP, DB, SMTP, Graph, Exchange, CalDAV, WebSocket, or fetch calls were found in source. |
| Wasm target safety | PASS | Runtime is `wasm32-wasip1`; crate is `cdylib` and uses no native-only crates, OS threads, or C bindings. |
| Secret handling | PASS | No credential reads or token storage paths found. Subscription input rejects token/password/client-secret style fields. |
| Host-effect boundary | PASS | Mutations return host-effect plans and require declared effects before constructing payloads. |
| Human UI present | PASS | `synapp.app.json` registers `ui/dist/index.html`; UI surfaces calendar, event, scheduling, invitation, sharing, subscription, settings, reminder, offline, and activity workflows. |

## OWASP Review

| Area | Status | Notes |
| --- | --- | --- |
| A01 Broken Access Control | PASS | Prior invite-to-write bypass is fixed; reviewed operations use context permission checks. |
| A02 Cryptographic Failures | PASS | No raw secrets are emitted; subscription token/password-like fields are rejected. |
| A03 Injection | PASS | Free-text inputs have bounds; no shell or interpreter execution exists. |
| A04 Insecure Design | PASS | Host-effect planning is explicit; sensitive operations require confirmation. |
| A05 Security Misconfiguration | PASS | App manifest capabilities align with the tool surface and permission gates. |
| A06 Vulnerable Components | PASS | Rust dependencies are limited to `serde` and `serde_json`; UI dependencies are React/Vite. |
| A07 Identification/Auth Failures | PASS | `context.user_id` is required and validated before use. |
| A08 Software/Data Integrity | N/A | No calendar catalog artifact exists yet, so package hash/signature checks are deferred. |
| A09 Logging/Monitoring Failures | PASS | Errors are structured and avoid sensitive details. |
| A10 SSRF | PASS | `subscribe_calendar` rejects raw URLs and forwards only host-safe refs. |

## Residual Risks

- Catalog publication integrity is not assessed because no `calendar-management` catalog entry exists yet. Add `catalog/apps/calendar-management.v1.json` with verified package and manifest hashes before release publication.
- Static review did not execute commands in the auditor session. Developer reported successful `cargo test`, `cargo build --release --target wasm32-wasip1`, and UI build after the fixes.
- The UI package uses React/Vite and previously reported two moderate npm audit findings in the Vite dependency tree. These are non-blocking for Synapp sandbox compliance but should be reviewed before release hardening.

## Final Verdict

FINAL VERDICT: PASSED

The calendar-management app passes Stage 5 security review after rework. Prior security-gating findings are fixed, regression coverage exists, and the implementation aligns with Synapp dual-interface and sandbox requirements.
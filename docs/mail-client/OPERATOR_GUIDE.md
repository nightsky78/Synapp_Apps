# Mail Client — Operator Guide

**App:** Mail Client v1.0.0  
**Artifact:** `mail_client.wasm`  
**Target:** `wasm32-wasip1`  
**Audience:** Platform engineers, DevOps, and on-call operators  

---

## Prerequisites

| Requirement | Version / Detail |
|---|---|
| Rust toolchain | stable (1.78 or later recommended) |
| Wasm target | `wasm32-wasip1` (install via `rustup target add wasm32-wasip1`) |
| Synapp Host Broker | Any version supporting WASI Preview 1 and platform effects |
| `wasm-objdump` (optional) | From `wabt` package — for export verification |
| `wasmtime` or `wasmer` (optional) | Required only for running native Wasm unit tests |

Install the Wasm target if not already present:

```bash
rustup target add wasm32-wasip1
```

---

## Build Instructions

### Standard Release Build

```bash
cd apps/MailClient
cargo build --release --target wasm32-wasip1
```

This produces: `apps/MailClient/target/wasm32-wasip1/release/mail_client.wasm`

Expected properties after a clean build:

| Property | Expected Value |
|---|---|
| File size | ~129 KB |
| Format | WebAssembly (wasm) binary module version 0x1 (MVP) |
| LTO | Enabled |
| Symbol stripping | Enabled |
| Optimisation level | `z` (size-optimised) |

### Build Profile

The `Cargo.toml` release profile is configured for minimal binary size:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

Do not override these settings for production builds. Debug builds (without `--release`) will produce a significantly larger binary not suitable for deployment.

### Ecosystem Build Script

**Note:** `scripts/build_all.sh` is currently broken. It targets `wasm32-wasi`, which is a deprecated name rejected by Rust 1.78+. Until this is fixed, build each app individually using the command above.

To fix the script:

```bash
# In scripts/build_all.sh, change line 6 from:
TARGET="wasm32-wasi"
# to:
TARGET="wasm32-wasip1"
```

This fix is tracked as blocker B-01 in the QA report.

---

## Deployment Checklist

Run these checks after every build before promoting to a production environment.

### 1. Binary Presence

```bash
ls -lh apps/MailClient/target/wasm32-wasip1/release/mail_client.wasm
```

Confirm: File exists and is approximately 129 KB. A significantly smaller file indicates an empty or stripped build. A significantly larger file may indicate debug symbols were included.

### 2. Binary Format

```bash
file apps/MailClient/target/wasm32-wasip1/release/mail_client.wasm
```

Expected output:

```
mail_client.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

### 3. Export Verification

All 8 functions must be exported. Use `wasm-objdump` from the `wabt` package:

```bash
wasm-objdump -x apps/MailClient/target/wasm32-wasip1/release/mail_client.wasm | grep "Export"
```

Or use `strings` as a lighter alternative:

```bash
strings apps/MailClient/target/wasm32-wasip1/release/mail_client.wasm \
  | grep -E "^(read_emails|get_email|draft_email|send_email|list_folders|move_email|search_emails|get_agent_permissions)$"
```

Expected: All 8 function names present. If any are missing, the build is incomplete or the `#[no_mangle]` attribute was removed from the export.

### 4. Schema Loaded

Confirm `apps/MailClient/plugin.json` is registered in the broker's schema registry. The schema version in `plugin.json` must match the deployed binary version. If the schema is not loaded, the broker cannot route AI tool calls to the binary.

### 5. Smoke Test

After deployment, invoke `get_agent_permissions` through the broker with:

```json
{
  "user_id": "test_user",
  "permission": "read"
}
```

Expected response:

```json
{
  "permission_level": "read",
  "user_id": "test_user",
  "agent_id": "...",
  "granted_at": <timestamp>,
  "expires_at": 0
}
```

A malformed or empty response indicates the binary is not loading correctly or the host is not passing parameters properly.

---

## Troubleshooting

### Build Failures

**Error: `toolchain 'stable-...' does not support target 'wasm32-wasi'`**  
Cause: You or a script used the old target name.  
Fix: Use `--target wasm32-wasip1` (note the `p1` suffix).

**Error: `error[E0463]: can't find crate for 'std'`**  
Cause: The `wasm32-wasip1` target is not installed.  
Fix: `rustup target add wasm32-wasip1`

**Error: linker errors involving C libraries**  
Cause: A dependency has been added that uses native C bindings incompatible with Wasm.  
Fix: Check `Cargo.toml` for new dependencies. Any crate requiring `cc`, `cmake`, or linking native libraries will break the Wasm build. Replace with a pure-Rust alternative or remove the dependency.

**Build succeeds but binary is unexpectedly large (> 300 KB)**  
Cause: `strip = true` may have been removed from the release profile, or LTO is disabled.  
Fix: Verify `Cargo.toml` profile settings match those documented above.

---

### Permission Errors at Runtime

**Symptom:** Functions return `{ "err": { "code": "PermissionDenied", ... } }` unexpectedly.  
Cause: The host broker is injecting a lower permission level than expected, or the agent profile has not been granted the required level.  
Steps:
1. Call `get_agent_permissions` to confirm the actual permission level in effect.
2. Check the agent's profile in the broker's IAM configuration.
3. Verify the broker is correctly passing `permission` as a function parameter, not omitting it.
4. An empty or unrecognised permission string defaults to `none` by design — confirm the string matches one of `"none"`, `"read"`, `"draft"`, `"send"` exactly (case-sensitive).

**Symptom:** `move_email` returns `PermissionDenied` for agents with `draft` permission.  
Cause: `move_email` requires `send` permission when moving to any folder other than DRAFTS. Moving to TRASH, ARCHIVE, INBOX, or SENT all require `send`.  
Fix: Grant the agent `send` permission, or restrict move operations to the DRAFTS folder when running under `draft` permission.

---

### Effect Failures

**Symptom:** Function returns `{ "err": { "code": "InternalError", ... } }`.  
Cause: A platform effect (e.g., `QueryEmailStore`, `SendEmail`, `CreateDocument`) failed on the broker side.  
Steps:
1. Check the Synapp Broker logs for the corresponding effect execution error.
2. Confirm the broker's email store is reachable and the user's mailbox exists.
3. For `send_email` failures: Confirm the platform SMTP service is available. The Wasm module itself does not perform SMTP — the failure is in the broker's effect executor.
4. Retry transient failures via the broker's retry mechanism. The Wasm module does not retry internally.

**Symptom:** `send_email` returns `DraftAlreadySent`.  
Cause: Expected — this is the idempotency guard. The draft was already dispatched successfully. The duplicate call was correctly rejected.  
Action: No action required if the first send succeeded. If the first send failed silently and the draft is stuck, check broker effect logs for the `CommitEmail` operation.

**Symptom:** `draft_email` returns `DraftNotFound` on a subsequent `send_email` call.  
Cause: Drafts expire after 7 days. If the `draft_id` is older than 7 days, it has been purged by the platform.  
Fix: Create a new draft and retry the send workflow.

---

### Schema / Routing Issues

**Symptom:** AI agent cannot discover or invoke Mail Client functions.  
Cause: `plugin.json` is not loaded in the broker registry, or the binary path is incorrect.  
Steps:
1. Confirm `plugin.json` is registered and syntactically valid JSON.
2. Confirm the binary path in the registry points to the correct `mail_client.wasm`.
3. Verify the `executionInterface.exports` entries in `plugin.json` match the actual Wasm exports exactly (case-sensitive function names).

---

## Monitoring and Observability

The Mail Client Wasm module is stateless and produces no logs directly. All observability is broker-side.

**Recommended metrics to track at the broker:**

| Signal | What to Watch |
|---|---|
| Effect execution latency | `QueryEmailStore` and `SearchEmailIndex` p95 latency — spikes indicate email store degradation |
| `PermissionDenied` rate | Elevated rate may indicate misconfigured agent profiles or a permission regression after a deploy |
| `InternalError` rate per function | Sudden increase indicates an effect executor failure, not a Wasm defect |
| `DraftNotFound` on `send_email` | High rate indicates agents are holding draft IDs beyond the 7-day expiry window |
| Binary load failures | Any failure to instantiate `mail_client.wasm` in the Wasm runtime indicates a binary or runtime compatibility issue |

**Tracing:** Instrument the broker to emit a trace span per function invocation. Include `function_name`, `user_id`, `permission`, and the resulting `ok`/`err` code. The Wasm module itself does not emit spans.

---

## Rollback Procedures

The Mail Client is stateless. The Wasm binary holds no data. Rolling back is safe and immediate.

### Steps to Roll Back

1. **Redeploy the previous binary:** Replace `mail_client.wasm` in the platform's module registry with the prior version.

2. **Restore the schema:** If `plugin.json` was updated alongside the binary, restore the previous `plugin.json` version in the broker registry. Schema and binary must be kept in sync.

3. **Verify the rollback:** Run the smoke test (`get_agent_permissions`) against the rolled-back binary to confirm it loads and responds correctly.

4. **No data migration required:** Because the app is stateless and all data lives in platform storage, there is no data to roll back. Email records, drafts, and folder structures are unaffected by a Wasm binary swap.

### What Cannot Be Rolled Back Automatically

- Emails that were sent via `send_email` during the faulty version's operation cannot be recalled. Platform mail delivery is external and irreversible.
- Drafts created during the faulty version remain in platform storage and are unaffected by the rollback.

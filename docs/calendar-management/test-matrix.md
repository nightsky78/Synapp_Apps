# Calendar Management Test Matrix

Stage: 6 - Testing  
App: `apps/first-party/calendar-management`  
Date: 2026-05-08  
Verdict: PASS, with documented release risks

## Scope

This matrix validates the first-party `calendar-management` Synapp app across the required Stage 6 surfaces:

- Rust planner behavior and contract tests.
- `wasm32-wasip1` release build.
- React/Vite human UI install and production build.
- Repository build script validation.
- `plugin.json`, `synapp.app.json`, and Rust export-name alignment.
- VS Code diagnostics for the new app and docs.
- Residual release risks and push gate status.

## Command Results

| Check | Command | Result | Evidence |
| --- | --- | --- | --- |
| Rust unit tests | `cd apps/first-party/calendar-management && cargo test` | PASS | 12 tests passed; 0 failed; finished in 0.01s. |
| Wasm release build | `cd apps/first-party/calendar-management && cargo build --release --target wasm32-wasip1` | PASS | Finished release profile for `wasm32-wasip1`. |
| UI dependency install | `cd apps/first-party/calendar-management/ui && npm ci` | PASS WITH NOTE | Added 63 packages and audited 64 packages. Reported 2 moderate vulnerabilities; no forced upgrade was run. |
| UI production build | `cd apps/first-party/calendar-management/ui && npm run build` | PASS | Vite 5.4.21 built 32 modules; generated `dist/index.html`, CSS, and JS assets. |
| Repo build validation | `cd /home/johannes/projects/Synapp_apps && scripts/build_all.sh` | PASS | Built `calendar-management`, `dummy-plugin`, and `mail-client`; script ended with `Build complete.` |
| Manifest parse and contract alignment | Node validation script against `plugin.json`, `synapp.app.json`, and `src/lib.rs` | PASS | Runtime alignment, entrypoint alignment, 38 plugin exports, 38 semantic tools, 38 app manifest tools, Rust `export_operation!` macros, and `free_string` source check all passed. |
| Compiled Wasm export table inspection | Checked `wasm-objdump`, `wasm-tools`, and `llvm-objdump` | LIMITED | No supported local Wasm export inspection tool was installed, so compiled export-table inspection could not be independently performed. Source-level and manifest-level export alignment passed. |
| VS Code diagnostics | `get_errors` for `apps/first-party/calendar-management` and `docs/calendar-management` | PASS | No errors found. |

## Rust Test Coverage

| Feature Area | Coverage | Result |
| --- | --- | --- |
| Export envelope and context validation | Every export rejects missing context. | PASS |
| Permission enforcement | Every export rejects missing permission; valid contract input accepted. | PASS |
| Host-effect availability | Required host effects are checked and unavailable effects return structured errors. | PASS |
| Host-effect planning | Mutating operations include stable idempotency keys. | PASS |
| Permission boundaries | Invite permission remains additive and does not grant unrelated privileged operations. | PASS |
| Event draft validation | Meeting attendee requirements and validation details are reported. | PASS |
| Recurrence | Validation and preview behavior are deterministic. | PASS |
| Suggested meeting times | Busy blocks are avoided when scoring suggestions. | PASS |
| Unknown enum handling | Unsupported planner enum values are rejected before host effects are planned. | PASS |
| Subscription safety | Raw provider fields, raw URLs, and sensitive subscription fields are rejected; sanitized host refs are forwarded. | PASS |

## Interface Coverage Matrix

| Interface | Covered Features | Validation Evidence | Result |
| --- | --- | --- | --- |
| Human UI | Calendar workspace, view switching, calendar/sidebar controls, event detail/editor, scheduling assistant, invitations, calendars/sharing/subscription, settings, reminders, offline state, activity, and bulk actions. | `npm ci` and `npm run build` passed; UI bridge maps human actions to the 38 operation names. | PASS |
| Wasm execution interface | 38 exported JSON ABI operations plus `free_string`; planner/validator behavior for reads, validation, mutation planning, recurrence, free/busy planning, suggestions, rooms, attendees, invitations, categories, sharing, reminders, preferences, offline, and audit. | `cargo test`, `cargo build --release --target wasm32-wasip1`, and source export alignment passed. | PASS |
| Semantic interface | `plugin.json` execution exports and semantic tools. | JSON parsed; 38 execution exports match 38 semantic tools and Rust export macros. | PASS |
| Synapp app manifest | Runtime, Wasm entrypoint, tools, capabilities, and UI entrypoint. | JSON parsed; runtime and entrypoint match `plugin.json`; 38 app tools match plugin exports. | PASS |
| Repository build policy | All app crates build with the repository `wasm32-wasip1` script. | `scripts/build_all.sh` passed. | PASS |
| Security boundary | No direct provider/network/database/AI execution was tested by behavior; subscription sanitization and host-effect planning are covered by Rust tests. | Rust tests passed and source/manifest contracts use host effects for side effects. | PASS |

## Manifest And Export Alignment Details

The manifest validation checked these invariants:

- `plugin.json.executionInterface.wasmTarget` equals `synapp.app.json.runtime`: `wasm32-wasip1`.
- `plugin.json.executionInterface.wasmPath` equals `synapp.app.json.entrypoint`: `target/wasm32-wasip1/release/calendar_management.wasm`.
- `plugin.json` execution exports match `plugin.json` semantic tools: 38 expected exports.
- `plugin.json` execution exports match `synapp.app.json` tools: 38 expected exports.
- `plugin.json` execution exports match Rust `export_operation!` macros: 38 expected exports.
- Rust source contains the declared `free_string` deallocator export.

## Generated Artifacts Validated

| Artifact | Status | Notes |
| --- | --- | --- |
| `target/wasm32-wasip1/release/calendar_management.wasm` | PRESENT AFTER BUILD | Built by Cargo release target. |
| `ui/dist/index.html` | PRESENT AFTER BUILD | Referenced by `synapp.app.json` as UI entrypoint. |
| `ui/dist/assets/index-BydKmWQm.css` | PRESENT AFTER BUILD | Vite CSS output, 11.23 kB reported. |
| `ui/dist/assets/index-DIUcuCpg.js` | PRESENT AFTER BUILD | Vite JS output, 185.74 kB reported. |

## Residual Risks

| Risk | Severity | Status | Release Note |
| --- | --- | --- | --- |
| NPM audit findings | Medium | Open | `npm ci` reported 2 moderate vulnerabilities. Per testing constraints, no forced dependency upgrades were run. Review with normal dependency policy before release hardening. |
| Compiled Wasm export-table inspection | Low/Medium | Tooling unavailable | No local `wasm-objdump`, `wasm-tools`, or `llvm-objdump` was available. Source-level and manifest-level alignment passed, but binary export inspection remains unverified. |
| Host integration | Medium | Not exercised in this local stage | UI local stub and Rust planner tests passed. End-to-end Synapp Broker host-effect execution was not available in this workspace. |
| Push gate | Medium | Blocked by unrelated dirty worktree changes | Existing unrelated modifications were detected outside `calendar-management`, including `apps/first-party/mail-client/synapp.app.json`, `catalog/apps/mail-client.v1.json`, and `scripts/package_app.sh`. No push was performed to avoid touching unrelated work. |

## Release Decision

Stage 6 local testing passes for `calendar-management`: Rust tests, Wasm build, UI install/build, repository build script, manifest/export alignment, and diagnostics all passed. Release readiness is acceptable for local pipeline progression with the residual risks above documented.

GitHub push was not performed because the repository contains unrelated dirty changes outside the calendar app. A push should be done only after the unrelated worktree state is isolated, committed separately, or explicitly approved for inclusion.
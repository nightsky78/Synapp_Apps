# Mail Client - Stage 6 Test Matrix & Release Evidence

**Date:** 2026-05-08  
**App:** `apps/first-party/mail-client`  
**Version:** 2.1.0  
**Runtime target:** `wasm32-wasip1`  
**UI entrypoint:** `ui/dist/index.html`  
**Engineer:** Testing Agent  
**Final verdict:** **PASSED**

The Stage 6 re-run passes Rust tests, Wasm target build, UI build, package generation, package content validation, catalog validation, repository build validation, static contract/security scans, diagnostics, and browser smoke. The prior release blocker is closed: `packages/mail-client-2.1.0.tar.gz` now includes the declared Wasm entrypoint, `ui/dist/index.html`, and UI assets.

## 1. Commands And Results

| Area | Command | Result | Evidence |
| --- | --- | --- | --- |
| Rust tests | `cd apps/first-party/mail-client && cargo test` | PASS | 14 tests passed, 0 failed. |
| Wasm build | `cargo build --release --target wasm32-wasip1` | PASS | Release build completed for `mail_client v2.1.0`. |
| UI build | `cd apps/first-party/mail-client/ui && npm run build` | PASS | Vite built `dist/index.html`, `dist/assets/index-BwtDAwAY.css`, and `dist/assets/index-DtxKF_5Q.js`. |
| Catalog validation | `./scripts/validate_catalog.sh` | PASS | `Validated 3 catalog apps`. |
| Repo build validation | `./scripts/build_all.sh` | PASS | Built `dummy-plugin` and `mail-client` for `wasm32-wasip1`. |
| Package generation | `./scripts/package_app.sh apps/first-party/mail-client` | PASS | Produced `packages/mail-client-2.1.0.tar.gz` with SHA-256 `9ce19321a38004d2c3e12d6f4edbecc0bf6efc2fc7b068b6055658af81a9b720`. |
| Package contents | `tar -tzf packages/mail-client-2.1.0.tar.gz \| sort` | PASS | Archive contains `target/wasm32-wasip1/release/mail_client.wasm`, `ui/dist/index.html`, `ui/dist/assets/index-BwtDAwAY.css`, and `ui/dist/assets/index-DtxKF_5Q.js`. |
| Declared entrypoint check | Node static check against `synapp.app.json` and tar contents | PASS | Declared Wasm entrypoint and declared UI entrypoint are both present; `ui_asset_count=2`. |
| Package hash | `sha256sum packages/mail-client-2.1.0.tar.gz` | PASS | Hash matches `catalog/apps/mail-client.v1.json`: `9ce19321a38004d2c3e12d6f4edbecc0bf6efc2fc7b068b6055658af81a9b720`. |
| Contract comparison | Node static comparison of Rust exports, `plugin.json`, source manifest, and catalog manifest | PASS | 44 Rust exports, 44 plugin exports, 44 source manifest tools, 44 catalog manifest tools; `EXPORT_SYNC_PASS`. |
| UI schema contract | Static source/catalog manifest scan | PASS | `UI_SCHEMA_SCAN_PASS`; no published `ui_schemas.main` or legacy UI schema object. |
| Source security scan | `rg` over Rust/UI source and manifests for unsafe rendering, direct network/DB, AI SDK terms | PASS | `SOURCE_SECURITY_SCAN_PASS`; no unsafe source rendering, direct network/DB, or embedded AI SDK/LLM logic found. |
| Credential schema scan | `rg '"(password\|access_token\|refresh_token\|client_secret\|smtp_password\|imap_password\|api_key)"\\s*:' ...` | PASS | `RAW_CREDENTIAL_SCHEMA_SCAN_PASS`; schemas publish opaque host refs rather than raw credential fields. |
| Security re-audit | `docs/mail-client/qa-security-report.md` | PASS | Security report states `FINAL VERDICT: PASSED`. |
| VS Code diagnostics | `get_errors` on Rust source and UI source | PASS | No errors reported for `src/lib.rs`; no UI diagnostics surfaced. |
| Browser smoke | Vite preview at `http://127.0.0.1:4173/` | PASS | First screen rendered the inbox workspace with `Synapp Mail`, Compose, account selector, search, Inbox message list, reading pane placeholder, Refresh, and Settings. It did not open on a settings form. |

## 2. Feature And Interface Coverage Matrix

| Feature / Interface | Rust Tests | Wasm Build | Manifest / Plugin | React UI | Package Artifact | Result |
| --- | --- | --- | --- | --- | --- | --- |
| Inbox-first workspace | N/A | N/A | `ui.entrypoint` points to React build; no published `ui_schemas.main` | Browser smoke shows inbox workspace first | UI entrypoint and assets included | PASS |
| No-account account setup | Account setup exports covered by Rust tests | PASS | Account setup tools present | Account setup is available through Settings/account flow and no-account state | UI bundle included | PASS |
| Mail read workflows | `existing_read_emails_workflow_is_preserved` and permission tests pass | PASS | `list_accounts`, `list_mailboxes`, `read_emails`, `search_emails`, `get_email`, `get_thread`, `get_unread_count` aligned | Inbox list/search/reading surfaces present | Wasm and UI included | PASS |
| Draft/send/schedule workflows | Send permission tests pass | PASS | Draft, send, reply, forward, schedule, cancel tools aligned | Compose and send/schedule controls present | Wasm and UI included | PASS |
| Organize/folders/rules workflows | Organize permission tests pass | PASS | Move, delete, archive, folder, and rules tools aligned | Bulk actions, folder manager, and rules manager present | Wasm and UI included | PASS |
| User account setup and migration | Account setup, connection test, preferences, and migration tests pass | PASS | Account/preferences/identity/migration tools aligned | Settings sections present | Wasm and UI included | PASS |
| Agent permission review | Permission matrix tests pass | PASS | `mail:*` capabilities and permission context present | Agent permissions/activity settings section present | Wasm and UI included | PASS |
| Security isolation | Static tests and source scans pass | PASS | Host effects only; no direct network/DB/AI logic found | No unsafe source rendering found | Package contains required files only | PASS |

## 3. Contract Validation Detail

| Check | Result | Evidence |
| --- | --- | --- |
| Rust export count | PASS | 44 `export_operation!` exports discovered in `src/lib.rs`. |
| `plugin.json` execution exports | PASS | 44 entries; exact name set matches Rust. |
| `synapp.app.json` tools | PASS | 44 entries; exact name set matches plugin exports. |
| Catalog manifest tools | PASS | 44 entries; exact name set matches source manifest. |
| Wasm target | PASS | Cargo build succeeded for `wasm32-wasip1`; manifest/plugin target paths use `target/wasm32-wasip1/release/mail_client.wasm`. |
| Main route settings schema removed | PASS | Published source and catalog manifests do not contain `ui_schemas.main` or a `ui_schemas` object. |
| UI entrypoint declared | PASS | Source manifest and catalog manifest declare `ui.entrypoint: ui/dist/index.html`. |
| UI entrypoint packaged | PASS | Package archive includes `ui/dist/index.html`. |
| UI assets packaged | PASS | Package archive includes `ui/dist/assets/index-BwtDAwAY.css` and `ui/dist/assets/index-DtxKF_5Q.js`. |
| Raw credential schema fields | PASS | No published schema properties named `password`, `access_token`, `refresh_token`, `client_secret`, `smtp_password`, `imap_password`, or `api_key`. |
| Opaque credential references | PASS | `secret_input_ref`, `secret_ref`, `oauth_connection_ref`, and `authorization_ref` are used as host-managed opaque refs. |

## 4. Security And Isolation Evidence

| Area | Result | Notes |
| --- | --- | --- |
| Unsafe React HTML source usage | PASS | No `dangerouslySetInnerHTML`, `innerHTML`, `document.write`, `eval`, or `new Function` matches in `ui/src`. |
| AI logic isolation | PASS | No source matches for OpenAI, LangChain, LLM, or embedded AI-routing logic. |
| Direct network / database isolation | PASS | No source matches for `std::net`, `TcpStream`, `UdpSocket`, `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, PostgreSQL, Redis, MySQL, or SQLite. |
| Wasm host-effect boundary | PASS | Rust code returns host-effect plans for IMAP/SMTP/OAuth/persistence/scheduling instead of opening protocol connections. |
| Secret handling | PASS | Tests validate raw secret rejection/redaction; schemas publish opaque refs only. |
| Package script guardrails | PASS | Generated package includes declared Wasm/UI entrypoints and fails closed when required entrypoints are missing. |

## 5. Blockers And Residual Risks

| ID | Severity | Status | Description | Disposition |
| --- | --- | --- | --- | --- |
| B-01 | HIGH | CLOSED | Prior package omitted `ui/dist/index.html` and UI assets. | Fixed. Re-run package contains declared UI entrypoint and assets. |
| R-01 | LOW | OPEN | Browser smoke used local Vite preview with app stubs, not a full Synapp Host Broker install. | Add host-loader E2E coverage when the broker package install path is available. |

No release-blocking risks remain from this Stage 6 re-run.

## 6. Regression Tests To Add

| Test | Purpose | Current Evidence |
| --- | --- | --- |
| Package contents test | Assert every manifest `entrypoint` and `ui.entrypoint` path exists inside the generated tarball, and assert the UI asset directory is non-empty. | Manual Node/tar check passed. |
| Package hash/catalog test | Assert regenerated package SHA-256 matches catalog `package_sha256`. | `sha256sum` matched catalog. |
| Manifest main route test | Assert `mail-client` source and catalog manifests do not publish `ui_schemas.main`. | `UI_SCHEMA_SCAN_PASS`. |
| Export sync test | Parse Rust `export_operation!` names and compare them with `plugin.json`, `synapp.app.json`, and catalog tool names. | `EXPORT_SYNC_PASS`. |
| Credential schema test | Assert schemas do not publish raw credential property names and only allow opaque host secret/OAuth refs. | `RAW_CREDENTIAL_SCHEMA_SCAN_PASS`. |
| Security static scan | Source-only scan for unsafe HTML rendering, direct network/DB APIs, and AI SDK/LLM strings. | `SOURCE_SECURITY_SCAN_PASS`. |
| Inbox first-screen smoke | Assert first UI screen is inbox workspace or account setup, not a settings form. | Vite preview browser smoke passed. |
| Packaged UI load E2E | Install/open packaged Mail Client and assert `ui/dist/index.html` loads from the package. | Not available in current environment; covered by package contents check until host-loader E2E exists. |

## 7. Release Decision

**Final verdict: PASSED.**

The packaging blocker is closed, the test matrix is current, and all critical Stage 6 checks pass. No GitHub push was performed; push was not required for this re-run.

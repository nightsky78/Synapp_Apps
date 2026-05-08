# Agent Entry Point

Use this file as the top-level entry point for the Synapp app development pipeline.

## Pipeline Orchestrator

**Primary entry point:** `.github/agents/pipeline.agent.md`

The Pipeline agent autonomously orchestrates the full app development workflow. Invoke it directly when starting new app features or bug fixes.

## Role Agents

Supporting agents invoked by the Pipeline:

- `.github/agents/ux-spec.agent.md` — Defines product UX scope, human and agent workflows, and acceptance criteria before architecture
- `.github/agents/architect.agent.md` — Defines app architecture, function contracts, and plugin.json schema mapping
- `.github/agents/ui-designer.agent.md` — Designs user-facing UI and interaction flows
- `.github/agents/developer.agent.md` — Implements Rust/Go Wasm code and synchronizes plugin.json
- `.github/agents/security-auditor.agent.md` — Audits code for OWASP Top 10, Wasm sandbox compliance, and security vulnerabilities
- `.github/agents/testing.agent.md` — Executes unit/integration tests and validates deployment

## Pipeline Execution Order

The Pipeline agent orchestrates stages sequentially:

```
UX-Spec → Architect → UI-Designer → Developer → QA-Security → Testing
```

If any stage fails, the Pipeline reroutes to fix loops (up to 3 retries) before halting or escalating.

## Mandatory Rules

- Follow `.github/copilot-instructions.md` strictly — all Dual-Interface rules apply
- Keep function signatures and `plugin.json` JSON schemas synchronized in every change
- Enforce `wasm32-wasi` or `wasm32-unknown-unknown` compilation target compatibility
- Zero AI logic inside app runtime code
- All exported functions must have corresponding UI surfaces for human interaction

## Quick Start

**To build a new app or feature:**
```
@Pipeline [feature description and scope]
```

**To resolve a GitHub issue:**
```
@Pipeline resolve GitHub issue [issue number]
```

The Pipeline agent will take it from there.

## Validation

Run the pipeline check:
```bash
/home/johannes/projects/Synapp_apps/.github/agents/run_pipeline_check.sh
```

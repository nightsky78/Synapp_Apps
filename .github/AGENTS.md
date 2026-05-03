# Agent Entry Point

Use this file as the top-level entry point for the Synapp role pipeline.

Pipeline config:
- .github/agents/pipeline.yaml
- .github/agents/run_pipeline_check.sh

Role agents:
- .github/agents/architect.agent.md
- .github/agents/ui-designer.agent.md
- .github/agents/developer.agent.md
- .github/agents/qa-security-tester.agent.md
- .github/agents/testing.agent.md
- .github/agents/techwriter.agent.md

Execution order:
1. architect
2. ui-designer
3. developer
4. qa-security-tester
5. testing
6. techwriter

Mandatory rules:
- Follow .github/copilot-instructions.md.
- Keep function signature changes synchronized with plugin.json schema updates.
- Enforce Wasm-target-safe implementation.

Validation:
- Run /home/johannes/projects/Synapp_apps/.github/agents/run_pipeline_check.sh

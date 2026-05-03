# Synapp Agent Pipeline

This directory defines the default multi-role agent pipeline for this repository.

Files:
- pipeline.yaml: Stage order, required artifacts, and stage gates.
- architect.agent.md: Architecture and contract definition.
- ui-designer.agent.md: UX and UI behavior specification.
- developer.agent.md: Implementation and schema synchronization.
- qa-security-tester.agent.md: Security and policy compliance review.
- testing.agent.md: Test execution and evidence.
- techwriter.agent.md: Documentation and release artifacts.
- run_pipeline_check.sh: Validates that required pipeline files exist and prints stage order.

How to use:
1. Start with architect.
2. Continue stage by stage in pipeline.yaml.
3. Do not skip gates.
4. If a gate fails, return to the responsible prior stage.
5. Run ./\.github/agents/run_pipeline_check.sh to validate pipeline integrity.

Required policy source:
- .github/copilot-instructions.md

Minimum completion criteria:
- Wasm export and plugin.json schema consistency is verified.
- Security and test evidence is documented.
- Documentation artifacts are produced.

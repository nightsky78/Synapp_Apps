# Synapp Apps Architecture Rules
1. The Dual-Interface Rule: Every app must expose its logic via Wasm exports (Execution Interface) and document it strictly in a local plugin.json (Semantic Interface).
2. No AI Logic in Apps: Wasm apps must NEVER contain LLM prompts, OpenAI SDKs, or AI routing logic. They are dumb executors of business logic.
3. Strict Wasm Compilation: All Rust/Go code must be written to compile to the wasm32-wasi or wasm32-unknown-unknown target.
4. State Isolation: Apps must not open direct network connections to databases. They must use the gRPC/WASI interfaces provided by the Synapp Host Broker.
5. Synchronized Commits: Whenever you modify a function signature in the source code, you MUST concurrently update the corresponding JSON Schema in the plugin.json.

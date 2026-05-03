# Synapp App Ecosystem: Copilot Instructions & Architecture Guide

## 1. Project Context & Platform Vision
You are acting as the core developer for the **Synapp App Ecosystem**. Synapp is an Omni-Channel, AI-Native Application Platform. Our fundamental goal is to solve the "AI Divide" where companies currently build separate tools for human employees and AI agents.

In Synapp, we build an app exactly once. 
* The apps are completely agnostic to their user. They do not know if a human clicked a button in a React UI, or if an autonomous AI agent decided to invoke a tool.
* To achieve this, the Synapp Host Broker acts as the middleman. It reads the capabilities of our apps, exposes them to the AI, and routes the execution back to our apps.
* Because the platform handles all AI routing and RAG (Retrieval-Augmented Generation) memory observation, the apps themselves must remain incredibly lightweight, secure, and completely void of AI logic.

## 2. The Dual-Interface Architecture
Every application in this repository must be built using our strict Dual-Interface paradigm:
* **The Execution Interface:** The compiled `.wasm` binary containing pure, highly performant business logic (e.g., `Calendar`).
* **The Semantic Interface:** A `plugin.json` manifest that sits alongside the source code. This acts as an AI Tool Manifest, describing the Wasm functions in plain English and JSON Schema so the AI broker understands how to use them.

## 3. Strict Development Rules
Whenever you generate code, plan a feature, or refactor logic in this repository, you must strictly adhere to the following constraints:

1. **The Dual-Interface Rule:** Every app must expose its logic via Wasm exports (Execution Interface) and document it strictly in a local `plugin.json` (Semantic Interface). One cannot exist without the other.
2. **Zero AI Logic in Apps (Dumb Executors):** Wasm apps must NEVER contain LLM prompts, OpenAI SDKs, LangChain logic, or AI routing mechanisms. They are dumb, highly efficient executors of business logic. The platform handles the intelligence; the app handles the execution.
3. **Strict Wasm Compilation:** All source code (Rust or Go) must be written exclusively to compile to the `wasm32-wasip1` (WASI Preview 1) or `wasm32-unknown-unknown` target. Do not use libraries that rely on native OS threads or C-bindings that cannot compile to WebAssembly.
4. **State & Network Isolation:** Apps must not open direct raw TCP/UDP network connections to databases (like PostgreSQL or Redis). They must remain perfectly sandboxed. If an app needs to persist data, it must use the gRPC/WASI host interfaces provided by the Synapp Broker.
5. **Synchronized Commits & Schemas:** The JSON Schema in the `plugin.json` is the unbreakable contract. Whenever you modify a function signature, add a parameter, or change a return type in the source code, you MUST concurrently update the corresponding JSON Schema in the `plugin.json`.

## 4. Your Workflow
When asked to build or modify an app:
1. **Design First:** Always ensure the `plugin.json` schema is defined before writing the implementation.
2. **Implement:** Write the Rust/Go code adhering to the `wasm32-wasip1` constraints.
3. **Verify:** Ensure the code is self-contained and the Wasm exports match the Semantic Interface exactly.
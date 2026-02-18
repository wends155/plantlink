# 🗺️ Project Context: PlantLink

> **AI Instructions:** This file is the Source of Truth. Update this file during the **Phase 4: Summarize** stage of the TARS workflow.

---

## 🏗️ System Overview
* **Goal:** A flow-based programming environment for IoT and Industrial Automation.
* **Core Stack:** Rust 2024 (Backend), Svelte Flow (Frontend), Rhai (Scripting).
* **Architecture Pattern:** Multi-crate Workspace (CLI, Web, Runtime, Core) + SvelteKit UI.

---

## 💻 Environment & Constraints
* **Host OS:** Windows (Non-Admin)
* **Shell Environment:** BusyBox (via Scoop) / PowerShell (use `;` for chaining)
* **Toolchain:** MSVC, Rustup (latest), Node.js v20+
* **Deployment:** Single-binary release capability with embedded assets.
* **Strict Rules:**
    1. No `sudo`/Admin commands.
    2. Scripts must be `#!/bin/sh` (BusyBox compatible).
    3. Use `Makefile` for standard workflows (`run`, `build-release`, `clean`).
    4. **Shell Syntax**: Use `;` for command chaining in PowerShell; use `&&` ONLY in BusyBox/sh scripts.

---

## 📍 Current State (Recursive Summary)
*This section is updated by the Architect after every successful implementation.*

### 🛠️ Recent Changes (Last 3 Cycles)
3.  **2026-02-11 (Project Config):** Configured `code-index` project root to `c:\Users\WSALIGAN\code\plantlink` and built deep index for symbol extraction (69 files).
4.  **2026-02-11 (Project Audit):** Conducted a deep architectural scan and generated `project_summary_report.md` covering crate roles, protocol status, and UI structure.
5.  **2026-02-18 (Architecture Remediation):** Replaced legacy `docs/ARCHITECTURE.md` with a fully compliant root `architecture.md` and added `todo.md` to track `spec.md` creation.
6.  **2026-02-18 (Spec Creation):** Created root `spec.md` with behavioral contracts for all 4 workspace crates (core, runtime, web, cli), covering data models, state machines, REST/WebSocket integration points, and CLI contracts.

### 🧩 Active Components & APIs
* `plantlink-core`: Shared data types (MessagePayload, DataValue) and protocol markers.
    * `src/lib.rs`: Core traits and payload definitions.
    * `src/mqtt.rs`, `src/modbus.rs`, `src/nats.rs`: Protocol-specific data structures.
* `plantlink-runtime`: Flow execution engine.
    * `src/lib.rs`: Engine logic and node orchestration.
    * `src/nodes/`: Concrete node implementations (Rhai scripts, MQTT, etc.).
* `plantlink-web`: API and UI backend.
    * `src/lib.rs`: Axum routes, WebSocket state, and asset embedding.
* `plantlink-cli`: Orchestration and entry point.
    * `src/main.rs`: Bootstraps the runtime and web server.
* `ui`: SvelteKit frontend.
    * `src/lib/nodeDefinitions.js`: Single source of truth for node UI metadata.

---

## 📜 Decision Log (The "Why")
*Records why specific paths were taken to prevent circular reasoning in future "Think" phases.*

* **Initial Design:** Chose **Rust** for the runtime to ensure high performance and reliability in industrial environments.
* **Initial Design:** Chose **Rhai** for scripting due to its familiar syntax and easy integration with Rust.

---

## 🚧 Technical Debt & Pending Logic
* **Known Issues:** None identified currently.
* **Next Steps:** Populate `context.md` after subsequent tasks to maintain state.

---

## 🧪 Tooling & Scripts
*Manual and automated scripts for development and verification.*

### 🛠️ Primary Tools
* **Makefile**: The central workflow orchestrator.
    * `make run`: Builds UI and launches CLI in dev mode.
    * `make check`: Runs `cargo check` for fast feedback.
    * `make verify`: Executes `scripts/verify.sh` for the full quality gate.
    * `make build-release`: Creates the production binary in `target/release/`.
* **scripts/verify.sh**: A strict quality gate running `fmt`, `clippy`, and `test`.
* **context7 (MCP)**: Specialized documentation server for Rust crates and industrial protocols (Modbus, NATS, etc.).
* **code-index (MCP)**: In-memory symbol indexer for fast file discovery, symbol extraction, and code pattern searching.
* **rust-mcp (MCP)**: Specialized Rust toolchain integration for workspace management, dependency analysis, and error diagnosis.
* **git-mcp (MCP)**: Automated version control server for atomic commits and repository history tracking.

### 🧪 Verification Commands
```bash
# Full Quality Gate
make verify

# Fast Linting
make clippy

# Dev Loop
make dev      # UI dev server
make run      # Full stack
```
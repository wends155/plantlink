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
109. **2026-03-31 (IDE-safe Git Status):** Encapsulated the non-interactive git status credential setup behind a `Makefile` target (`git-status`) to bypass IDE execution blockers on chained powershell commands. Updated the `toolcheck.md` workflow to mandate the usage of `make git-status` instead of the legacy `$env:GCM_INTERACTIVE='never'` approach. Verified using `make verify`.
108. **2026-03-31 (Theming Infrastructure Refactor):** Refactored the frontend theme management system from a binary toggle to an extensible N-ary registry. Introduced a frozen `THEMES` metadata registry in `ui/src/lib/stores/theme.js` to decouple theme names from visual modes. Implemented an automatic `.dark` class application for all dark-variant themes (including the newly activated **Nord** theme) to maintain compatibility with Tailwind `dark:` utilities. Updated `InnerFlowEditor.svelte` to utilize a derived `colorMode` store, ensuring binary compatibility for SvelteFlow's rendering logic. Verified 100% "Zero-Exit" status across all unit tests and the full `make verify` pipeline.
107. **2026-03-31 (Documentation Refresh):** Updated the global `README.md` to reflect recent technical milestones. Added explicit mentions of the Bearer Token Authentication middleware for REST endpoints under the new "Security" feature slice, and clarified that the `make run` development orchestration handles automatic local token injection for seamless hot-reloading.
106. **2026-03-31 (Documentation Refresh):** Synchronized the user documentation in the `docs/` repository folder (`API.md`, `RHAI_SCRIPTING.md`) with the latest system capabilities. Documented the new `Authorization: Bearer` token requirements for the REST endpoints. Refreshed Rhai scripting artifacts to outline the new `DataValue::Bytes` placeholder passthrough optimization and clarify that script `Compilation Error` states are now explicitly caught at flow deployment rather than runtime execution.

### 🧩 Active Components & APIs
* `plantlink-core`: Shared data types (MessagePayload, DataValue) and protocol traits.
* `plantlink-runtime`: Flow execution engine with structured concurrency support.
* `plantlink-web`: API and UI backend; embeds Auth middleware and `EventCache` aggregator.
* `ui`: SvelteKit frontend; utilizes the `THEMES` registry for CSS-variable driven styling and SvelteFlow orchestration.

---

## 📜 Decision Log (The "Why")
*Records why specific paths were taken to prevent circular reasoning in future "Think" phases.*

* **Initial Design:** Chose **Rust** for the runtime to ensure high performance and reliability in industrial environments.
* **Initial Design:** Chose **Rhai** for scripting due to its familiar syntax and easy integration with Rust.
* **Error Handling:** Adopted a "No Silent Failure" policy for the runtime; `NodeContext::send_output` now returns `Result<()>` to ensure callers handle or log delivery failures.
* **Resilience:** Implemented exponential backoff for MQTT event loops to prevent event loop crashes and ensure automatic recovery from connection errors.
* **Trait-Based Mocking:** Transitioned protocol drivers to traits to enable isolated node-level testing.
* **Theming N-ary Registry:** Opted for a registry-based store rather than a binary toggle to support multiple high-contrast and brand-colored themes (e.g., Nord) without bloating store logic. 
* **Tailwind Compatibility:** Decided to force-apply `.dark` for any theme marked with `colorMode: 'dark'` to ensure existing `dark:` Tailwind utilities function correctly without per-theme selector updates.

---

## 🚧 Technical Debt & Pending Logic
* **E2E Theme Persistence**: Need automated Playwright coverage for system-preference matching and persistence across sessions.
* **Modbus Resiliency**: `ModbusTcpClient` requires an exponential-backoff loop to handle network drops.
* **Rhai Validation**: `NodeFactory` needs to return `Result` to allow pre-deployment validation of scripts.

---

## 🧪 Tooling & Scripts
*Manual and automated scripts for development and verification.*

### 🛠️ Primary Tools
* **Makefile**: The central workflow orchestrator.
    * `make run`: Builds UI and launches CLI in dev mode.
    * `make verify`: Executes the strict 4-gate quality pipeline (fmt, clippy, test, ast-grep).
* **context7 (MCP)**: Specialized documentation server for Rust crates and industrial protocols.
* **code-index (MCP)**: In-memory symbol indexer for fast file discovery and symbol extraction.

### 🧪 Verification Commands
```bash
# Full Quality Gate
make verify

# UI Unit Tests
npm run test:unit src/lib/stores/theme.test.js
```
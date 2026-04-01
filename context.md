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

### 🛠️ Recent Changes (Last 6 Cycles)
119. **2026-04-01 (Artifact Persistence Directives):** Added explicit `write_to_file` commands to Step 4 of the `/issue`, `/audit`, and `/plan-making` workflows to ensure byte-for-byte TARS artifact compliance.
118. **2026-04-01 (Workflow Governance Cleanup):** Pruned the already-remediated AI Workflow Governance task from `todo.md`.
117. **2026-03-31 (Knowledge Base Restructuring):** Replaced the project-specific `plantlink_system_architecture` Knowledge Item with a generalized `rust_ecosystem_patterns` KI to codify cross-project technical standards. Added explicit structural recipes for async coordination (`TaskTracker`), testing boundaries (`mockall`, `testcontainers`, `wiremock`, `insta`, `proptest`), and Zero-Exit governance (Clippy deny profiles, `ast-grep` rules) to synchronize standard engineering practices.
116. **2026-03-31 (Validated Tech Debt Remediation):** Hardened the `ModbusTcpClient` actor with exponential backoff for reconnect attempts (1s-60s) and fixed a critical cancellation bug in `NatsSubNode` background listeners. Pruned stale tech debt tracking. Verified 100% pass rate via `make verify`.
115. **2026-03-31 (Runtime Observability Hardening):** Implemented UI-visible error reporting for `NatsSubNode` broker lookup failures and introduced a consecutive-error circuit breaker in the `RuntimeEngine` task loop (threshold: 5) to halt broken nodes and prevent log spam. Verified with TDD-first unit tests and the full `make verify` pipeline.
114. **2026-03-31 (NatsSubNode Lifecycle Fix):** Implemented `start()` hook for `NatsSubNode` to ensure static subscriptions are successfully established upon flow deployment instead of waiting for a dynamic port-0 ping. Extracted subscription logic from `receive()` into a reusable, idempotent `subscribe_and_listen()` helper method. Integrated `mockall` into tests to verify zero-exit behavior independently of a live NATS broker. Verified using TDD and full `make verify` pipeline.
113. **2026-03-31 (Runtime Lifecycle & Security Fixes):** Fixed a critical "Zombie Flow State" bug in `RuntimeEngine::update_flow` by ensuring `stop_flow().await` is called during partial initialization failures. Resolved a payload collision vulnerability in `RhaiNode` by embedding the `MessagePayload.id` (UUID) directly into the binary serialization placeholder string. Verified zero-exit via new TDD test cases and `make verify`.
112. **2026-03-31 (Architecture Remediation):** Implemented `mockall::automock` for `PubSubClient` and `ModbusClient` in `plantlink-core` to fulfill architectural mockability claims. Pruned stale tech debt from `architecture.md` regarding Rhai script validation. Verified via 13 unit tests and full `make verify` pipeline.
111. **2026-03-31 (Documentation Sync):** Synchronized `spec.md` behavioral contracts with the current `HEAD` (hash `320a1c9`) and significantly enhanced `Rustdoc` comments for core `plantlink-runtime` traits and structs. Standardized documentation across `base.rs`, `lib.rs`, and `mod.rs` with `# Arguments`, `# Returns`, and `# Errors` sections for better API discoverability. Verified 100% test pass rate via `make verify`.
110. **2026-03-31 (Runtime Lifecycle Fix):** Implemented automatic "running" status emission in `BaseNodeAdapter::start()` to ensure all `SimpleNode` implementations (e.g., `ConsoleNode`, `NatsNode`) provide UI feedback without manual status calls. Resolved by emitting a generic "Running" status before delegating to `on_start()`, allowing specific nodes to provide more descriptive overrides. Verified with a new TDD test case in `base.rs` and the full `make verify` pipeline.
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
* **Protocol Integrations**: Need formal integration test coverage for MQTT, NATS, and Modbus using `testcontainers-rs` or `docker-compose` to verify full driver-to-broker round trips in CI.

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

---

## 📜 Session History
### 2026-04-01: Workflow Governance Cleanup
* **Feature:** Remove remediated AI Workflow Governance tech debt item.
* **Changes:** Removed the stale tracking item from `todo.md` related to enforcing explicit artifact URI placeholders in `plan-making.md`.
* **New Constraints:** None.
* **Pruned:** The "AI Workflow Governance" technical debt entry is now fully resolved.

### 2026-03-31: Global unwrap() Safety Enforcement
* **Feature:** Add `clippy::unwrap_used = "deny"` and `clippy::expect_used = "warn"` to workspace `Cargo.toml`.
* **Changes:** Promoted the "No Crashes" lint from a soft AST-grep warning to a hard compiler-enforced build failure. Added blanket `#[allow]` attributes to 12 test modules to maintain test ergonomics, and applied 2 targeted `#[allow(clippy::expect_used)]` annotations to provably infallible production `.expect()` sites (`build.rs` npm commands, `main.rs` signal handlers).
* **New Constraints:** Developers are strictly forbidden from using `.unwrap()` in production code. Use safe variants (`unwrap_or`, `unwrap_or_else`) or proper `Result` propagation instead.
* **Pruned:** The reliance on the AST-grep `unwrap-in-production` rule as the sole `.unwrap()` enforcement mechanism.

### 2026-03-31: Runtime Observability Hardening
* **Feature:** UI-visible Error Reporting and Node Circuit Breaker
* **Changes:** Added `ctx.emit_error()` to `NatsSubNode` for missing broker scenarios; implemented a `MAX_CONSECUTIVE_ERRORS` (5) halt mechanism in the engine task loop.
* **New Constraints:** Nodes that fail 5 times consecutively will be halted and require a flow redeploy to reset.
* **Pruned:** Silent NATS broker lookup failures and infinite node error loops.

### 2026-03-31: Zombie Flow & Rhai Security Fixes
* **Feature:** Zombie Flow State and Rhai Placeholder Security Fixes
* **Changes:** Added `stop_flow().await` fallback to `update_flow` for cleanup safety; hardened Rhai binary serialization with a UUID nonce payload placeholder.
* **New Constraints:** None.
* **Pruned:** The "Zombie Flow State" and "Rhai Placeholder Collision" technical debt items have been fully resolved.

### 2026-03-31: Runtime Audit & Documentation Cleanup
* **Audit Validation:** Conducted a cross-validation of the `plantlink-runtime` architecture and code review reports. Confirmed 7/10 findings.
* **Corrected Overstated Findings:** Downgraded NATS driver resource leak and cache staleness claims. Verified that the atomic `update_flow()` lifecycle (stop -> recreate) inherently prevents these issues.
* **Documentation Sync:** Resolved stale technical debt entries in `architecture.md` and `context.md` relating to Rhai script validation (already implemented in `dc85251`).
* **Test Hygiene:** Removed a misleading "assertion will fail" comment in `rhai.rs` for a test that now correctly passes.
* **Quality Gate:** Verified all changes pass the full `make verify` pipeline.
 
### 2026-03-31: NatsSubNode Lifecycle Fix
* **Feature:** Implementing `start()` for `NatsSubNode` to subscribe on deployment without dynamic ping.
* **Changes:** Extracted `subscribe_and_listen()` from `receive()`, implemented `start()` utilizing the helper, and integrated tests with `MockPubSubClient` to assert zero-exit success.
* **New Constraints:** The underlying architecture still lacks comprehensive Engine topological startup ordering, which forces `NatsSubNode::start()` failures to degrade to warnings instead of failing flow deployments.
* **Pruned:** N/A.

### 2026-03-31: Runtime Quick-Win Remediations
* **BaseNodeAdapter Fix:** Implemented automatic `emit_running("Running")` in the adapter's `start()` method. This ensures that nodes implementing `SimpleNode` have their "running" status reported to the UI even if they don't explicitly call `emit_running`. 
* **Architecture Docs Update:** Updated `architecture.md` §5 (Module Boundaries) to explicitly document the internal sub-modules of `plantlink-runtime` (`base`, `registry`, `rhai`, `nats`, `inject`, `console`), addressing a key finding from the architectural audit.
* **Deferred Technical Debt:** Formally deferred complex refactors (NATS mockability, Rhai performance/security) to dedicated future cycles to maintain focus on the current stability phase.
* **Verification:** Confirmed 52+ tests and doctests pass with zero-exit in the quality pipeline.

### 2026-03-31: Validated Tech Debt Remediation
* **Feature:** Modbus Backoff and NATS Lifecycle Hardening
* **Changes:** Added exponential backoff to `ModbusTcpClient` (1s to 60s); added `TaskTracker` test to `NatsSubNode` and fixed its cancellation response; pruned stale tech debt from roadmap.
* **New Constraints:** None.
* **Pruned:** "Modbus Resiliency", "Rhai Script Validation", and "E2E Theme Persistence" technical debt items.

### 2026-03-31: NatsBrokerNode Clarification
* **Feature:** Pruned `NatsBrokerNode` task tracking tech debt
* **Changes:** Marked `NatsBrokerNode` task tracking as inapplicable in `todo.md` because `async-nats` multiplexing relies purely on drop semantics rather than explicit `TaskTracker` integration. Checked off the Structured Concurrency checklist.
* **New Constraints:** None.
* **Pruned:** The final legacy entry from the Structured Concurrency hardening checklist.
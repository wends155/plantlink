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

### 🛠️ Recent Changes (Initial Context)
1.  **2026-02-11:** Initialized `context.md` by extracting details from `README.md` and project structure.
2.  **N/A:** Previous local state not tracked in this context fragment.

### 🧩 Active Components & APIs
* `plantlink-core`: Shared data types (MessagePayload, DataValue).
* `plantlink-runtime`: Flow execution engine with node implementations (Rhai, NATS, etc.).
* `plantlink-web`: Axum-based web server and WebSocket handler for UI interaction.
* `plantlink-cli`: Main entry point and orchestration layer.
* `ui`: SvelteKit frontend for visual flow editing.

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

## 🧪 Verification Commands
*Standard commands the Executor must run to pass the Quality Gate.*

```bash
# Linting & Verification
make check

# Build & Run (Dev)
make run

# Build Release
make build-release

# Cleanup
make clean
```
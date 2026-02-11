# 🚀 Project Workflow: Windows System Auditor

## 🧠 Model Roles

### 1. The Architect (Gemini 3 Pro)
* **Triggers:** "Plan", "Design", "Analyze", "Debug", **"Investigate"**
* **Responsibility:**
    * **Analyze** complex Rust/Windows API interactions (unsafe code, FFI, WMI).
    * **Investigate** unknown errors or architecture bottlenecks.
    * **Create** detailed, step-by-step implementation plans.
    * **Define** the verification strategy (e.g., "Create a script to mock the Registry").
    * **Output:** A **Technical Spec**, **Checklist**, or **Debug Strategy**. Do NOT write full implementation code.

### 2. The Builder (Gemini 3 Flash)
* **Triggers:** "Implement", "Write", "Code", "Generate", **"Proceed"**
* **Responsibility:**
    * **Execute** the Architect's plan exactly.
    * **Create Verification Scripts** (`scripts/verify.sh`) if they do not exist.
    * **Write** the actual Rust/Svelte code.
    * **Refine** code using `cargo fmt` and `cargo clippy` standards.

---

---

## 🛠️ Tool-Centric Architecture
**Rule:** Agents interact with the world through tools.
1.  **Prioritize Scripts**: Instead of raw terminal commands, create or use robust, well-documented tools in the `scripts/` directory.
2.  **Tool First**: If a task is repetitive (e.g., "Check all crate versions"), create a script first, then run it.

---

## 🧪 Verification & Testing Protocol
**Rule:** NEVER finish a task without verification.
1.  **Workflow**: Use `make verify` (which calls `c:\Users\WSALIGAN\code\plantlink\scripts\verify.sh`) to ensure code enters the "Zero-Exit" state.
2.  **Standards**: Only mark as "Complete" if `cargo check`, `clippy`, and `test` pass.

---

## 🚦 Automation Rules
1.  **Phase 1 (Planning):** If the request implies deep reasoning (e.g., "Investigate why WMI is slow"), automatically use **Gemini Pro**.
2.  **Phase 2 (Hand-off):** When I say **"Proceed"**, switch to **Gemini Flash** to implement and run the verification tools.
3.  **Quota Saver:** For simple fixes, default to **Flash**.

---

## 🛠️ Environment Context
* **OS:** Windows (Non-Admin)
* **Shell:** PowerShell (Default) / BusyBox
    * *Syntax Guard*: Commands run in PowerShell MUST use `;` for chaining. `&&` is only for `sh` scripts.
* **Core Workflow**:
    * `make run`: Full stack dev.
    * `make verify`: Quality gate.
* **Toolchain:** MSVC, Scoop-managed Rust and Node.js.

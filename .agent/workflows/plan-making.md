---
description: How to create a high-quality implementation plan (Think Phase)
---

# Plan-Making Workflow

This workflow defines the standard process for creating implementation plans.
It enforces the Planning Gate and Think Phase of the TARS protocol.

## Prerequisites

> [!TIP]
> Run `pwsh -NonInteractive -Command "& '.agent/scripts/Load-Context.ps1' -Mode plan"` to gather all prerequisite context in one step.

- Read `architecture.md` (if present) for project-specific design, toolchain, and patterns.
- Read `.agent/rules/coding-standard.md` (if present) for language-specific coding standards.
- Read `.agent/rules/ipr.md` (if present) for implementation plan format and handoff rules.
- Read `context.md` (if present) for historical decisions and prior context.
- If a Report was produced by `/issue`, `/audit`, or `/feature`, use it as the **primary input** for Step 1. Do not re-investigate areas already covered.
- Confirm you are operating in **Planning mode** (no code edits allowed).

## Steps

### 1. Scope & Impact Analysis

Investigate the request before writing anything:

- **Identify affected files**: List every file/module that will be touched.
- **Map dependencies**: What depends on those files? What do they depend on?
- **Flag risks**: Security concerns, breaking changes, performance impacts.
- **Check for existing tests**: Search for test files related to the affected code.

#### MCP-Enhanced Analysis *(when available)*

If **Narsil MCP** is available, use it throughout planning:

**Investigation** (Step 1):

| Tool | Purpose |
|------|---------|
| `get_import_graph`, `get_dependencies` | Visualize what's affected by proposed changes |
| `find_circular_imports`, `check_cwe_top25`, `check_owasp_top10` | Catch structural or security risks early |
| `find_symbols`, `find_references`, `get_symbol_definition` | Understand interfaces before proposing changes |
| `find_unused_exports`, `find_dead_code` | Identify cleanup opportunities to include in the plan |

**Validation** (Step 2 — use results to support proposed changes):

| Tool | Purpose |
|------|---------|
| `get_symbol_definition` | Verify interfaces/types being modified exist as expected |
| `find_references` | Check blast radius of proposed signature changes |
| `check_dependencies` | Check for vulnerable deps before adding new ones |
| `find_similar_code` | Find existing patterns the plan should follow |

For **M/L tier plans**, the Architect **SHOULD** use `sequentialthinking` to break down complex changes, reason about ordering, and validate root cause coverage before drafting. For **S-tier plans**, skip it — the overhead isn't worth it.

### 2. Draft the Plan

Follow the plan format, revision protocol, and handoff rules defined in `.agent/rules/ipr.md`.

### 3. Sync task.md

After drafting the plan, synchronize `task.md` with the proposed changes:

```powershell
pwsh -NonInteractive -Command "& '.agent/scripts/Sync-TaskList.ps1' -Mode generate -PlanFile '<plan-path>'"
```

The script writes `task.md` to the same directory as the plan file.
If `task.md` already exists it will be overwritten. Run `-Mode validate`
afterwards to confirm alignment.

> [!WARNING]
> Do NOT skip this step. `task.md` must be aligned with the plan before
> requesting approval. Run `-Mode validate` to confirm exit code 0.

### 4. Self-Review Checklist

Before requesting approval, verify each item. Items marked 🤖 can be verified
with Narsil MCP or scripts; items marked 🧠 require LLM judgment.

**Scope & Coverage:**
- [ ] 🤖 All affected files are listed (verify with Narsil `find_references`)
- [ ] 🤖 Each change is broken into numbered, independently verifiable steps
- [ ] 🧠 Module boundaries defined (Owns / Does NOT own)
- [ ] 🧠 Interface contracts specified (signatures, invariants, error conditions)
- [ ] 🧠 Cross-module handshakes documented (caller/callee, data format, error propagation)
- [ ] 🧠 Code snippets included for non-trivial changes

**Compliance** (cross-reference each proposed change against these rules):

| Rule Source | Check |
|-------------|-------|
| GEMINI.md § Error Handling | New functions handle errors with what/where/why; no silent failures |
| GEMINI.md § Observability | Plan includes structured logging for significant operations |
| GEMINI.md § Testing | Test Plan covers all new/changed logic |
| GEMINI.md § Documentation | New public APIs will have doc comments |
| coding-standard.md *(if present)* | Error handling (§4.1), async (§4.2), patterns (§4.6), module org (§4.7), observability (§4.8), defensive programming (§4.9), prohibited patterns (§10) |
| architecture.md *(if present)* | Layout conventions, toolchain commands |

> [!CAUTION]
> If any proposed change cannot satisfy a rule, document the exception with
> justification in the Edge Cases & Risks section. Do not silently skip compliance.

**Process:**
- [ ] 🤖 No code was edited (Planning Gate enforced)
- [ ] 🧠 `context.md` consulted for historical decisions (if present)
- [ ] 🧠 Constraints clearly documented in Problem Statement
- [ ] 🤖 Dependencies researched — check with Narsil `check_dependencies`
- [ ] 🧠 Risks and edge cases documented
- [ ] 🧠 Mermaid diagram included for structural changes

**Integration:**
- [ ] 🤖 Report findings incorporated (if `/issue`, `/audit`, or `/feature` was run)
- [ ] 🤖 MCP tools used for investigation/analysis where available
- [ ] 🤖 task.md synced — `pwsh -NonInteractive -Command "& '.agent/scripts/Sync-TaskList.ps1' -Mode validate"` returns exit 0

### 5. Request Approval

Before requesting approval, run the pre-flight gate:

```powershell
pwsh -NonInteractive -Command "& '.agent/scripts/Sync-TaskList.ps1' -Mode preflight -PlanFile <plan-path>"
```

> [!CAUTION]
> The pre-flight gate MUST return exit 0 before requesting approval.
> If it fails, fix the issues and re-run. Do NOT skip this step.

End the plan with:

> 🛑 **Think Phase Complete.** Reply with **"Proceed"** to Act.

Do NOT proceed to implementation until the user explicitly approves.

### 6. Post-Approval Handoff

Once approved, follow **GEMINI.md §6 Handoff Protocol** for the full Act cycle.
After `/audit` passes, run `/update-doc` scoped to affected files, then summarize in `context.md` per GEMINI.md §8.

---
description: Validate tooling environment at session start (Session Bootstrap)
---

# Toolcheck Workflow

This workflow validates the agent's tooling environment at session start.
It checks MCP connectivity, repo indexing, toolchain health, workflow/script
availability, and identifies automation opportunities.

> [!IMPORTANT]
> Run this workflow at the start of every session to ensure all tools are
> operational before beginning work.

## Trigger

User invokes: `/toolcheck`

## Steps

### 1. Environment Scan (Script)

Run the companion script to check all mechanical items:

```powershell
pwsh -NonInteractive -Command "& '.agent/scripts/Check-Environment.ps1' -Mode scan"
```

This produces a structured report covering:
- Shell tools (git, rg, PowerShell version)
- Rust toolchain (rustc, cargo, clippy, rustfmt, rustup)
- Linkers (MSVC link.exe/cl.exe, gcc — including scoop conflict detection)
- Workflow files (8 required + 1 alternative group in `.agent/workflows/`)
- Script files (7 expected in `.agent/scripts/`)
- Repo root scripts (any .ps1, .sh, Makefile, justfile)
- Project detection (Cargo.toml, package.json, go.mod)

### 2. Diagnose & Fix

For each ❌ item in the scan report:

1. **Diagnose** — determine why the tool is missing or misconfigured.
   Common causes: not installed, wrong PATH order, missing component.

2. **Attempt user-space fix** *(if possible)*:

   | Issue | Fix |
   |-------|-----|
   | Missing clippy | `rustup component add clippy` |
   | Missing rustfmt | `rustup component add rustfmt` |
   | Missing rg | `scoop install ripgrep` (preferred) or `cargo install ripgrep` (slow, compiles from source) |
   | Scoop `link.exe` shadowing MSVC | Advise user to reorder PATH |
   | Missing workflow/script file | ⚠️ Cannot fix — warn user |

3. **Re-scan** — run `pwsh -NonInteractive -Command "& '.agent/scripts/Check-Environment.ps1' -Mode scan"` again to confirm the fix.

4. **If unfixable** — collect into warnings with:
   - What failed
   - Why it can't be auto-fixed (e.g., requires admin, requires manual install)
   - Recommended action for the user

> [!NOTE]
> Fixes are **user-space only** — no admin rights, no sudo.
> If a fix requires elevated privileges, warn the user and provide
> the exact command they would need to run manually.

### 3. MCP Connectivity

#### Narsil MCP

1. **Connectivity**: Call `list_repos` — if it returns, Narsil is connected.
2. **Repo validation**: Call `validate_repo` with the current project path.
3. **Indexing**: Call `reindex` to trigger a fresh index for the session.
4. **Status**: Call `get_index_status` to confirm indexing is complete and review enabled features (git, call-graph, persist, watch).
5. **Dynamic Scoping**: Note the `$RepoRoot` path from the environment scan summary. **You MUST use this path as the `path="..."` argument** in all subsequent Narsil tool calls to isolate your analysis to the current project, avoiding noise from the macro-workspace.

If Narsil is **not available**, note it as a warning — not a blocker.
Other workflows can fall back to manual investigation.

#### Sequential Thinking MCP

1. **Connectivity**: Call `sequentialthinking` with a simple diagnostic thought.
   - If it returns, Sequential Thinking is available.
   - If it errors, note as a warning.

#### Context7 MCP

1. **Connectivity**: Call `resolve-library-id` with a simple query.
   - If it returns, Context7 is available for documentation lookups.
   - If it errors, note as a warning.

### 4. Project Assessment

If **Narsil MCP** is connected and indexed, perform a project-level scan:

| Tool | Purpose |
|------|---------|
| `get_project_structure` | Understand repo layout and key files |
| `check_dependencies` | Scan for known vulnerable dependencies |
| `get_security_summary` | Overall security posture |

Report any critical vulnerabilities or structural issues found.

### 5. Automation Opportunities

If **Sequential Thinking MCP** is available, use `sequentialthinking` to analyze:

1. **Scan results** — are there patterns that could be automated?
2. **TODO/FIXME markers** — load via `pwsh -NonInteractive -Command "& '.agent/scripts/Load-Context.ps1' -Mode issue"` output.
3. **Project structure** — are there build scripts, CI configs, or Makefiles?
4. **MCP capabilities** — which Narsil tools could help with current project state?
5. **Script gaps** — are there repetitive tasks that need a new script?

If Sequential Thinking is **not available**, perform this reasoning inline.

### 6. Session Readiness Report

Produce the final structured report:

```markdown
## 🚀 Session Readiness Report

### Environment
| Tool | Status | Version/Details |
|------|--------|----------------|
| PowerShell | ✅/❌ | version (edition) |
| Git | ✅/❌ | version |
| Rust | ✅/❌ | version + edition |
| Linker | ✅/❌ | MSVC/GCC + conflict status |
| rg | ✅/❌ | version |

### MCP Servers
| Server | Status | Details |
|--------|--------|---------|
| Narsil | ✅/❌ | repos indexed, features enabled |
| Sequential Thinking | ✅/❌ | available/unavailable |

### Workflow Ecosystem
| Component | Status |
|-----------|--------|
| Workflows | N/M present |
| Scripts | N/M present |

### Fixes Applied
- [list of auto-fixes attempted and their results]

### ⚠️ Warnings
- [unfixable issues + recommended user actions]

### 🤖 Automation Opportunities
- [identified by Sequential Thinking analysis]
```

End with:

> ✅ **Session Ready.** All critical tools operational.

or:

> ⚠️ **Session Ready with warnings.** Review the warnings above.
> Non-critical issues documented — workflows will use fallback paths.

## Rules

1. **Always scan first** — never skip `Check-Environment.ps1`, even if "everything looks fine."
2. **Fix before warn** — attempt user-space fixes before escalating to the user.
3. **No admin** — all fixes must be user-space (rustup, cargo install, scoop).
4. **Always index** — trigger Narsil `reindex` for fresh data every session.
5. **Don't block** — unfixable issues are warnings, not blockers. Other workflows fall back to manual investigation.
6. **Report everything** — even passing items go in the report for the session record.

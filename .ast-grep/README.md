# ast-grep Rules for Rust Coding Standards

This directory contains `ast-grep` rules designed to mechanically enforce project coding standards (`coding-standard.md`) that `cargo clippy` cannot cover. 

These rules leverage ast-grep's AST-awareness and path-scoping to detect architectural violations like scattered configuration access or database queries outside of repository modules.

## How to Adopt in a Project

1. Install ast-grep:
   ```sh
   cargo install ast-grep
   ```
2. Copy `sgconfig.yml` to your project root (update `ruleDirs` to `.ast-grep/rules`).
3. Sync the `.ast-grep/` directory from the template repo.
4. Run the scan:
   ```sh
   sg scan
   ```

*(Workflows like `/audit` and `/build` will automatically detect `sgconfig.yml` and run `sg scan` as a 4th quality gate).*

## Provided Rules

| ID | Description | standard ref | Severity |
|---|---|---|---|
| `scattered-env-var` | Warns on `std::env::var` outside `config.rs` | §4.10 | Warning |
| `block-on-in-async` | Errors on `block_on` in an async context | §4.2 | Error |
| `raw-thread-spawn` | Warns on `std::thread::spawn`, preferring tokio | §10 | Warning |
| `hardcoded-url` | Hints on string literals starting with `http://` | §4.10 | Hint |
| `sqlx-outside-repo` | Warns on sqlx macros outside `repo` modules | §4.6.7 | Warning |
| `mutex-option-antipattern` | Hints `Mutex<Option<T>>` to prefer `OnceLock` | §4.6.5 | Hint |
| `arc-vec-antipattern` | Warns `Arc<Vec<T>>` to prefer `Arc<[T]>` | §4.3 | Warning |
| `raw-tokio-spawn` | Hints `tokio::spawn` to prefer structured concurrency | §4.2 | Hint |
| `wildcard-import` | Warns wildcard imports `use foo::*` | §4.7.4 | Warning |
| `missing-instrument` | Hints missing `#[instrument]` on `pub async` | §4.8 | Hint |
| `scattered-dotenv` | Warns `dotenv::dotenv()` outside `main` or `config` | §4.10 | Warning |
| `unwrap-in-production` | Warns `.unwrap()` in non-test code | §4.1 | Warning |

## Customizing Scope

Rules use the `ignores:` array to whitelist specific files (e.g., `tests/`, `config.rs`). 
If your project uses different module naming conventions, you should edit the `ignores:` arrays in the YAML files to match your project's geography.

### Ignoring False Positives

To ignore a finding inline, add an ast-grep comment above the line:
```rust
// ast-grep-ignore
let val = std::env::var("MY_VAR");
```

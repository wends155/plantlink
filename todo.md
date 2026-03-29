# PlantLink — TODO

## 🛡️ Documentation
- [x] Create `spec.md` (Behavioral Source of Truth per `GEMINI.md`)
  - [x] Module/Component Contracts for each crate
  - [x] Data Models (`MessagePayload`, `DataValue`, etc.)
  - [x] CLI Contracts (`plantlink-cli` arguments, flags, exit codes)
  - [x] Integration Points (WebSocket API, REST endpoints)
- [x] Synchronize `spec.md` with structured concurrency refactor (Cycle 85)

## 🏗️ Structured Concurrency (Zero-Exit Hardening)
- [/] Migrate nodes and tests to structured concurrency (`JoinSet`, `TaskTracker`)
  - [x] Refactor `RuntimeEngine` to use tracker-aware shutdown
  - [x] Implement line-level AST linter suppressions for legacy tests
  - [/] Add TDD `TaskTracker` validation tests for all spawning nodes
    - [x] `InjectNode` (timer tracking)
    - [ ] `NatsSubNode` (listener tracking)
    - [ ] `NatsBrokerNode` (driver handle tracking, if applicable)

## 🛠️ Technical Debt
- [ ] Implement robust error recovery for Modbus TCP reconnections
- [ ] Add Rhai script validation during flow deployment
- [ ] Add E2E Playwright test for Event Cache lag recovery

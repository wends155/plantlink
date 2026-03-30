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
- [x] Implement robust error recovery for Modbus TCP reconnections
- [ ] Add Rhai script validation during flow deployment
- [x] Add E2E Playwright test for Event Cache lag recovery
- [x] Harden Web Server (Auth, Heartbeats, Structured Concurrency)
- [ ] Protocol Integrations (High Value, Higher CI/CD Complexity): Our main selling point is IoT/SCADA connectivity. The current test suite doesn't actually test MQTT, NATS, or Modbus because we don't spin up brokers in the Makefile. The Gap: spec.md lists MqttDriver, NatsDriver, and ModbusTcpClient as needing integration test coverage. The Tradeoff: To test these in Playwright E2E or via make test-integration, we need to decide whether to introduce docker-compose or testcontainers-rs into the verification gate, or keep the CI lightweight for now.

## 🤖 AI Workflow Governance
- [ ] Update `.agent/workflows/plan-making.md` handoff template to explicitly embed markdown artifact URI placeholders. This mechanically forces the Architect to provide direct links to the plan and task files before prompting `Proceed`.

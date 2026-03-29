.PHONY: build-assets run test-e2e clean build-release dev preview toolcheck todos secrets doc-coverage doc-comments diff-last lint-ast sections log-summary log-lifecycle log-timings check-stubs verify

# Development server
dev:
	cd ui && npm run dev

# Build UI assets
build-assets:
	cd ui && npm install && npm run build

# Run full stack (dev)
run: build-assets
	PLANTLINK_AUTH_TOKEN="local-dev-token" cargo run -p plantlink-cli

# Production build
build-release: build-assets
	cargo build --release -p plantlink-cli

# Preview production build locally
preview:
	cd ui && npm run build && npm run preview

# Clean
clean:
	cargo clean
	rm -rf ui/dist ui/node_modules/.vite
	rm -f *.log *.txt

# Testing
test-unit:
	cd ui && npm run test:unit

test-integration:
	cd ui && npm run build
	pwsh -File scripts/start-backend.ps1
	cd ui && npm run test:integration
	pwsh -File scripts/stop-backend.ps1

test-e2e:
	cd ui && npm run test:e2e

test-e2e-ui:
	cd ui && npm run test:e2e:ui

# Verification
# Full verification pipeline (Gates 1-4)
# Used by /build and /audit workflows — one command, no chaining.
verify:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-features
	$(MAKE) lint-ast

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

# Gate 4: AST linting (ast-grep)
lint-ast:
	sg scan

check:
	cargo check

test:
	cargo test

# Environment toolcheck
toolcheck:
	git --version
	rg --version
	rustc --version
	cargo --version
	cargo clippy --version
	rustfmt --version
	rustup show
	git config credential.helper

# Scan for TODO/FIXME/HACK markers across source files
todos:
	rg -n -e "TODO" -e "FIXME" -e "HACK" --glob "*.rs" --glob "*.go" --glob "*.ts" --glob "*.js" --glob "*.svelte" --glob "*.py" . || true

# Scan for hardcoded secrets
secrets:
	rg -n -i -e "API_KEY" -e "SECRET" -e "PASSWORD" -e "TOKEN" --glob "!.git" --glob "!target" --glob "!*.lock" . || true

# Count public items per source file (doc coverage metric)
doc-coverage:
	rg -c -e "pub\s+fn\s+" -e "pub\s+struct\s+" -e "pub\s+enum\s+" -e "pub\s+trait\s+" -e "pub\s+type\s+" --glob "*.rs" . || true

# Count doc comment lines per source file
doc-comments:
	rg -c "\s*///" --glob "*.rs" . || true

# Show full patch of the most recent commit (safe: no ~ character)
diff-last:
	git log -1 -p

# List markdown section headings (for architecture audit)
sections:
	rg -n "## " $(FILE)

# Log analysis targets
log-summary:
	rg -c "TRACE|DEBUG|INFO|WARN|ERROR" logs/ || true
	rg -n "WARN|ERROR" logs/ --max-count 50 || true

log-lifecycle:
	rg -n "thread spawned|thread started|thread exiting|initialized|shutting down|dropping" logs/ || true
	rg -n "connection established|connection closed|reconnect|evict|cache hit|cache miss" logs/ || true

log-timings:
	rg -n "elapsed_ms=|duration_ms=|took [0-9]+ms|latency_ms=" logs/ || true
	rg -c "elapsed_ms=[0-9]+|duration_ms=[0-9]+|took [0-9]+ms" logs/ || true
	rg -o "elapsed_ms=[0-9]+" logs/ --no-filename || true

# Stub checking target
check-stubs:
	rg -n "STUB\(Phase" || true


.PHONY: build-assets run test-e2e clean build-release dev preview toolcheck todos secrets

# Development server
dev:
	cd ui && npm run dev

# Build UI assets
build-assets:
	cd ui && npm install && npm run build

# Run full stack (dev)
run: build-assets
	cargo run -p plantlink-cli

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
verify:
	sh ./scripts/verify.sh

verify-all: verify test-unit test-e2e

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

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

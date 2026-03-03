.PHONY: build-assets run test-e2e clean build-release dev preview

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

.PHONY: build-assets run test-e2e

build-assets:
	cd ui && npm install && npm run build

run: build-assets
	cargo run -p plantlink-cli

clean:
	cargo clean
	rm -f *.log *.txt

build-release:
	cd ui && npm install && npm run build
	cargo build --release -p plantlink-cli

test-e2e:
	cd ui && npm run test:e2e

test-e2e-ui:
	cd ui && npm run test:e2e:ui

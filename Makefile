.PHONY: build-assets run

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

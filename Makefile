.PHONY: test lint ci-test

# Local testing: assumes you are inside the nix-shell so DATABASE_URL is set.
test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings
	cargo fmt --all -- --check

# CI/CD Entrypoint: validates DB connection, runs tests, then generates a single report.
ci-test: lint
	@if [ -z "$$DATABASE_URL" ]; then \
		echo "Error: DATABASE_URL is not set for CI"; exit 1; \
	fi
	cargo test --workspace --locked

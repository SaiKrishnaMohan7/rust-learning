.DEFAULT_GOAL := help
.PHONY: help check build release run test test-v test-one watch fmt fmt-check lint lint-fix doc clean audit deny tools ci

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

# ---- build / run ----

check: ## Typecheck only, no codegen (fastest feedback loop)
	cargo check --all-targets

build: ## Debug build
	cargo build

release: ## Optimized build
	cargo build --release

run: ## Build and run (debug)
	cargo run

# ---- tests ----

test: ## Run all tests
	cargo test

test-v: ## Run tests, show println! output
	cargo test -- --nocapture

test-serial: ## Run tests one at a time (for flaky concurrency tests)
	cargo test -- --test-threads=1

test-one: ## Run tests matching NAME=  e.g. make test-one NAME=cache
	cargo test $(NAME) -- --nocapture

# ---- quality ----

fmt: ## Format code
	cargo fmt

fmt-check: ## Fail if code is unformatted
	cargo fmt --check

lint: ## Clippy with warnings as errors
	cargo clippy --all-targets -- -D warnings

lint-fix: ## Clippy with autofix where possible
	cargo clippy --fix --allow-dirty --allow-staged

ci: fmt-check lint test ## Everything CI would run

# ---- misc ----

doc: ## Build and open docs (includes dependencies)
	cargo doc --open

watch: ## Re-run `cargo check` on save (needs cargo-watch)
	cargo watch -x check

clean: ## Remove target/
	cargo clean

audit: ## Check dependencies for known vulnerabilities (needs cargo-audit)
	cargo audit

deny: ## Check licenses, bans, duplicate versions (needs cargo-deny)
	cargo deny check

tools: ## Install the optional cargo subcommands used above
	cargo install cargo-watch cargo-audit cargo-deny cargo-nextest cargo-expand
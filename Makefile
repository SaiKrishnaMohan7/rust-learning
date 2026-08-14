.DEFAULT_GOAL := help
.PHONY: help crates check build release run test test-v test-serial test-one watch fmt fmt-check lint lint-fix doc clean audit deny tools ci

# Set P=<crate> to scope a command to one workspace member.
#   make test P=cachelab   ->  cargo test -p cachelab
# Without P, commands run across the whole workspace.
PKG := $(if $(P),-p $(P),--workspace)

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "  Add P=<crate> to scope to one member, e.g. make test P=cachelab"

crates: ## List workspace members
	@cargo metadata --no-deps --format-version 1 | grep -o '"name":"[^"]*"' | cut -d'"' -f4

# ---- build / run ----

check: ## Typecheck only, no codegen (fastest feedback loop)
	cargo check $(PKG) --all-targets

build: ## Debug build
	cargo build $(PKG)

release: ## Optimized build
	cargo build $(PKG) --release

run: ## Build and run. REQUIRES P=  e.g. make run P=hello-async
	@if [ -z "$(P)" ]; then echo "run needs a crate: make run P=<crate>"; exit 1; fi
	cargo run -p $(P)

# ---- tests ----

test: ## Run tests (whole workspace, or P=<crate>)
	cargo test $(PKG)

test-v: ## Run tests, show println! output
	cargo test $(PKG) -- --nocapture

test-serial: ## Run tests one at a time (for flaky concurrency tests)
	cargo test $(PKG) -- --test-threads=1

test-one: ## Run tests matching NAME=  e.g. make test-one NAME=cache [P=cachelab]
	cargo test $(PKG) $(NAME) -- --nocapture

# ---- quality ----

fmt: ## Format code (always whole workspace)
	cargo fmt --all

fmt-check: ## Fail if code is unformatted
	cargo fmt --all --check

lint: ## Clippy with warnings as errors
	cargo clippy $(PKG) --all-targets -- -D warnings

lint-fix: ## Clippy with autofix where possible
	cargo clippy $(PKG) --fix --allow-dirty --allow-staged

ci: fmt-check lint test ## Everything CI would run

# ---- misc ----

doc: ## Build and open docs (includes dependencies)
	cargo doc $(PKG) --open

watch: ## Re-run `cargo check` on save (needs cargo-watch)
	cargo watch -x "check $(PKG) --all-targets"

clean: ## Remove target/ (shared across the workspace)
	cargo clean

audit: ## Check dependencies for known vulnerabilities (needs cargo-audit)
	cargo audit

deny: ## Check licenses, bans, duplicate versions (needs cargo-deny)
	cargo deny check

tools: ## Install the optional cargo subcommands used above
	cargo install cargo-watch cargo-audit cargo-deny cargo-nextest cargo-expand
# Makefile for Skill Engine
# Provides convenient commands for development and testing

.PHONY: help build test coverage clean dev docs-dev catalog-dev

# Default target
help:
	@echo "Skill Engine - Available Commands"
	@echo ""
	@echo "Building:"
	@echo "  make build          Build the project in debug mode"
	@echo "  make release        Build the project in release mode"
	@echo ""
	@echo "Testing:"
	@echo "  make test           Run all tests"
	@echo "  make test-unit      Run unit tests only"
	@echo "  make test-integration  Run integration tests"
	@echo "  make test-claude-bridge  Run Claude Bridge tests"
	@echo ""
	@echo "Documentation:"
	@echo "  make dev            Run docs + catalog dev servers (parallel)"
	@echo "  make docs-dev       Run docs dev server (localhost:5173)"
	@echo "  make catalog-dev    Run catalog dev server (localhost:3000)"
	@echo "  make docs-build     Build documentation site"
	@echo "  make catalog-build  Build catalog site"
	@echo ""
	@echo "Coverage:"
	@echo "  make coverage       Generate coverage report (all formats)"
	@echo "  make coverage-unit  Generate coverage for unit tests"
	@echo "  make coverage-html  Generate HTML coverage report"
	@echo "  make coverage-open  Generate and open HTML coverage report"
	@echo "  make coverage-check Verify coverage meets thresholds"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          Remove build artifacts"
	@echo "  make clean-coverage Remove coverage reports"

# Build targets
build:
	cargo build

release:
	cargo build --release

# Test targets
test:
	cargo test --workspace

test-unit:
	cargo test --workspace --lib

test-integration:
	cargo test --workspace --test '*'

test-claude-bridge:
	cargo test -p skill-cli --lib -- claude_bridge
	@if [ -f tests/claude_bridge/test-all.sh ]; then \
		./tests/claude_bridge/test-all.sh; \
	fi

# Coverage targets
coverage:
	cargo tarpaulin --workspace --out Html --out Xml --out Lcov

coverage-unit:
	cargo tarpaulin -p skill-cli --lib --out Html

coverage-integration:
	cargo tarpaulin -p skill-cli --test '*' --out Html

coverage-html:
	cargo tarpaulin --workspace --out Html

coverage-open: coverage-html
	@if command -v open > /dev/null; then \
		open tarpaulin-report.html; \
	elif command -v xdg-open > /dev/null; then \
		xdg-open tarpaulin-report.html; \
	else \
		echo "Coverage report generated: tarpaulin-report.html"; \
	fi

coverage-check:
	cargo tarpaulin --workspace --out Xml --fail-under 70

# Cleanup targets
clean:
	cargo clean

clean-coverage:
	rm -f cobertura.xml tarpaulin-report.html lcov.info
	rm -rf target/tarpaulin

# Development helpers
fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

check: fmt lint test

# CI simulation
ci: check coverage-check
	@echo "✓ CI checks passed"

# Documentation development targets
dev:
	@echo "Starting docs + catalog dev servers..."
	@echo "Docs:    http://localhost:5173/skill/"
	@echo "Catalog: http://localhost:3000"
	@trap 'kill 0' SIGINT; \
	(cd docs-site && VITE_CATALOG_URL=http://localhost:3000 npm run dev) & \
	(cd marketplace-web && npm run dev) & \
	wait

docs-dev:
	cd docs-site && VITE_CATALOG_URL=http://localhost:3000 npm run dev

catalog-dev:
	cd marketplace-web && npm run dev

docs-build:
	cd docs-site && npm run build

catalog-build:
	cd marketplace-web && npm run build

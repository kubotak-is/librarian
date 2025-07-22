# Librarian Development Makefile

.PHONY: help dev build quality quality-frontend quality-rust test clean

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

dev: ## Start development server
	npm run tauri dev

build: ## Build the application
	npm run tauri build

# Frontend quality checks
quality-frontend: ## Run frontend quality checks
	@echo "🔍 Running frontend quality checks..."
	@echo "📝 Checking TypeScript and Svelte..."
	npm run svelte-check
	@echo "📎 Running ESLint..."
	npm run lint
	@echo "🎨 Checking Prettier formatting..."
	npm run format:check
	@echo "✅ Frontend quality checks completed!"

# Rust quality checks
quality-rust: ## Run Rust quality checks
	@echo "🔍 Running Rust quality checks..."
	@echo "📝 Checking format..."
	cd src-tauri && cargo fmt -- --check
	@echo "📎 Running Clippy..."
	cd src-tauri && cargo clippy --all-targets -- -D warnings
	@echo "🧪 Running tests..."
	cd src-tauri && cargo test
	@echo "🏗️ Checking compilation..."
	cd src-tauri && cargo check --all-targets
	@echo "✅ Rust quality checks completed!"

quality: quality-frontend quality-rust ## Run all quality checks

# Formatting
format-frontend: ## Format frontend code
	npm run format

format-rust: ## Format Rust code
	cd src-tauri && cargo fmt

format: format-frontend format-rust ## Format all code

# Linting with fixes
lint-fix-frontend: ## Fix frontend linting issues
	npm run lint:fix

lint-fix: lint-fix-frontend format ## Fix all linting issues

# Testing
test-frontend: ## Run frontend tests
	npm test

test-rust: ## Run Rust tests
	cd src-tauri && cargo test

test: test-frontend test-rust ## Run all tests

# Build checks
build-check: ## Quick build verification
	npm run build
	cd src-tauri && cargo check --all-targets

# Clean
clean: ## Clean build artifacts
	rm -rf dist node_modules/.vite
	cd src-tauri && cargo clean

# Development helpers
install: ## Install dependencies
	npm install

update: ## Update dependencies
	npm update
	cd src-tauri && cargo update

# CI/CD helpers
ci-check: ## Run CI checks (no fixes)
	@echo "🚀 Running CI quality checks..."
	$(MAKE) quality
	$(MAKE) build-check
	@echo "✅ All CI checks passed!"

# Agent library helpers
mcp-start: ## Start MCP server for this repository
	@echo "Starting librarian MCP server..."
	npm run tauri dev &
	@echo "MCP server started. Use 'claude code add-mcp-server librarian-self --url http://localhost:9500' to connect"

mcp-test: ## Test MCP endpoints
	@echo "Testing MCP endpoints..."
	curl -X POST http://localhost:9500 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"prompts/list"}' | jq .

# Release management
release-patch: ## Create patch release (bug fixes)
	@echo "🏷️ Creating patch release..."
	gh workflow run version.yml -f version_type=patch

release-minor: ## Create minor release (new features)
	@echo "🏷️ Creating minor release..."
	gh workflow run version.yml -f version_type=minor

release-major: ## Create major release (breaking changes)
	@echo "🏷️ Creating major release..."
	gh workflow run version.yml -f version_type=major

release-status: ## Check release workflow status
	@echo "📊 Release workflow status:"
	gh run list --workflow=release.yml --limit=5

release-local: ## Build release locally (requires Rust)
	@echo "🏗️ Building local release..."
	npm run build
	npm run tauri build

prerelease: ## Create prerelease (alpha/beta)
	@echo "🏷️ Creating prerelease..."
	gh workflow run version.yml -f version_type=patch -f prerelease=true

# Distribution helpers  
dist-check: ## Check distribution readiness
	@echo "📦 Checking distribution readiness..."
	@echo "✅ GitHub Actions workflows:"
	@ls -la .github/workflows/
	@echo "✅ Package.json version:"
	@node -p "require('./package.json').version"
	@echo "✅ Documentation:"
	@ls -la README.md INSTALL_GUIDE.md DESIGNDOG.md
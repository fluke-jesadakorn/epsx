.PHONY: help make-admin unassign-admin

# Show help by default
.DEFAULT_GOAL := help

help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

make-admin: ## Make a user an admin (Usage: make make-admin email=user@example.com)
	@if [ -z "$(email)" ]; then \
		echo "Error: email parameter is required"; \
		echo "Usage: make make-admin email=user@example.com"; \
		exit 1; \
	fi
	bun run src/commands/make-admin.ts $(email)

unassign-admin: ## Remove admin rights from a user (Usage: make unassign-admin email=user@example.com)
	@if [ -z "$(email)" ]; then \
		echo "Error: email parameter is required"; \
		echo "Usage: make unassign-admin email=user@example.com"; \
		exit 1; \
	fi
	bun run src/commands/unassign-admin.ts $(email)

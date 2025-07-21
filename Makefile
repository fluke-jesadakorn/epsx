# EPSX Monorepo Makefile
# ===========================================

# Docker registry and project variables
REGISTRY := us-central1-docker.pkg.dev/epsx-449804/epsx
PROJECT_NAME := epsx

# Environment (default to development)
ENV ?= development

# Image names
FRONTEND_IMAGE := $(REGISTRY)/frontend
ADMIN_IMAGE := $(REGISTRY)/admin
BACKEND_IMAGE := $(REGISTRY)/backend

# Tag with timestamp
TAG_FILE := .docker-tag
TAG := $(shell cat $(TAG_FILE) 2>/dev/null || date +%Y%m%d-%H%M%S)

# Docker build platforms
PLATFORMS := linux/amd64

# Colors for output
RED := \033[31m
GREEN := \033[32m
YELLOW := \033[33m
BLUE := \033[34m
RESET := \033[0m

# ===========================================
# Development Commands
# ===========================================

.PHONY: help install dev dev-full build clean

help: ## Show this help message
	@echo "$(BLUE)EPSX Monorepo Commands$(RESET)"
	@echo "======================="
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

install: ## Install all dependencies
	@echo "$(YELLOW)Installing dependencies...$(RESET)"
	pnpm install

dev: ## Start development servers (frontend + admin)
	@echo "$(GREEN)Starting development servers...$(RESET)"
	pnpm dev

dev-full: ## Start all development servers including backend
	@echo "$(GREEN)Starting all development servers...$(RESET)"
	pnpm dev:all

build: ## Build all applications and packages
	@echo "$(BLUE)Building all packages and apps...$(RESET)"
	pnpm build

build-packages: ## Build only packages
	@echo "$(BLUE)Building packages...$(RESET)"
	pnpm build:packages

build-apps: ## Build only applications
	@echo "$(BLUE)Building applications...$(RESET)"
	pnpm build:apps

# ===========================================
# Quality Assurance
# ===========================================

.PHONY: lint lint-fix type-check test test-unit test-e2e format

lint: ## Run linting
	@echo "$(YELLOW)Running linters...$(RESET)"
	pnpm lint

lint-fix: ## Fix linting issues
	@echo "$(YELLOW)Fixing linting issues...$(RESET)"
	pnpm lint:fix

type-check: ## Run TypeScript type checking
	@echo "$(BLUE)Running type checks...$(RESET)"
	pnpm type-check

test: ## Run all tests
	@echo "$(GREEN)Running all tests...$(RESET)"
	pnpm test

test-unit: ## Run unit tests
	@echo "$(GREEN)Running unit tests...$(RESET)"
	pnpm test:unit

test-e2e: ## Run end-to-end tests
	@echo "$(GREEN)Running E2E tests...$(RESET)"
	pnpm test:e2e

format: ## Format code
	@echo "$(BLUE)Formatting code...$(RESET)"
	pnpm format

format-check: ## Check code formatting
	@echo "$(BLUE)Checking code formatting...$(RESET)"
	pnpm format:check

# ===========================================
# Cleanup Commands
# ===========================================

.PHONY: clean clean-cache clean-all

clean: ## Clean build artifacts
	@echo "$(RED)Cleaning build artifacts...$(RESET)"
	pnpm clean

clean-cache: ## Clean all caches
	@echo "$(RED)Cleaning caches...$(RESET)"
	pnpm clean:cache

clean-all: clean clean-cache ## Clean everything
	@echo "$(RED)Cleaning everything...$(RESET)"

# ===========================================
# Docker Commands
# ===========================================

.PHONY: docker-build docker-build-frontend docker-build-admin docker-build-backend docker-push docker-up docker-down

docker-build: docker-build-frontend docker-build-admin ## Build all Docker images

docker-build-frontend: build-packages ## Build frontend Docker image
	@echo "$(BLUE)Building frontend Docker image...$(RESET)"
	@echo $(TAG) > $(TAG_FILE)
	docker build \
		--platform $(PLATFORMS) \
		-t $(FRONTEND_IMAGE):$(ENV)-$(TAG) \
		-t $(FRONTEND_IMAGE):$(ENV)-latest \
		-f apps/frontend/Dockerfile \
		--build-context frontend=. \
		.

docker-build-admin: build-packages ## Build admin Docker image
	@echo "$(BLUE)Building admin Docker image...$(RESET)"
	@echo $(TAG) > $(TAG_FILE)
	docker build \
		--platform $(PLATFORMS) \
		-t $(ADMIN_IMAGE):$(ENV)-$(TAG) \
		-t $(ADMIN_IMAGE):$(ENV)-latest \
		-f apps/admin-frontend/Dockerfile \
		--build-context admin=. \
		.

docker-build-backend: ## Build backend Docker image
	@echo "$(BLUE)Building backend Docker image...$(RESET)"
	@echo $(TAG) > $(TAG_FILE)
	docker build \
		--platform $(PLATFORMS) \
		-t $(BACKEND_IMAGE):$(ENV)-$(TAG) \
		-t $(BACKEND_IMAGE):$(ENV)-latest \
		-f apps/backend/Dockerfile \
		apps/backend

docker-push: docker-build ## Build and push all Docker images
	@echo "$(GREEN)Pushing Docker images...$(RESET)"
	docker push $(FRONTEND_IMAGE):$(ENV)-$(TAG)
	docker push $(FRONTEND_IMAGE):$(ENV)-latest
	docker push $(ADMIN_IMAGE):$(ENV)-$(TAG)
	docker push $(ADMIN_IMAGE):$(ENV)-latest

docker-up: ## Start Docker services
	@echo "$(GREEN)Starting Docker services for $(ENV)...$(RESET)"
	docker-compose -f docker-compose.$(ENV).yml up -d

docker-down: ## Stop Docker services
	@echo "$(RED)Stopping Docker services for $(ENV)...$(RESET)"
	docker-compose -f docker-compose.$(ENV).yml down

docker-logs: ## View Docker logs
	@echo "$(BLUE)Viewing Docker logs for $(ENV)...$(RESET)"
	docker-compose -f docker-compose.$(ENV).yml logs -f

# ===========================================
# Environment Commands
# ===========================================

.PHONY: env-setup env-dev env-test env-prod

env-setup: ## Setup environment scripts
	@echo "$(YELLOW)Setting up environment scripts...$(RESET)"
	chmod +x scripts/setup-environment.sh

env-dev: env-setup ## Setup development environment
	@echo "$(GREEN)Setting up development environment...$(RESET)"
	./scripts/setup-environment.sh dev

env-test: env-setup ## Setup test environment
	@echo "$(YELLOW)Setting up test environment...$(RESET)"
	./scripts/setup-environment.sh test

env-prod: env-setup ## Setup production environment
	@echo "$(RED)Setting up production environment...$(RESET)"
	./scripts/setup-environment.sh prod

# ===========================================
# Workspace Commands
# ===========================================

.PHONY: workspace-graph workspace-outdated workspace-update

workspace-graph: ## Show workspace dependency graph
	@echo "$(BLUE)Generating workspace dependency graph...$(RESET)"
	pnpm workspace:graph

workspace-outdated: ## Check for outdated dependencies
	@echo "$(YELLOW)Checking for outdated dependencies...$(RESET)"
	pnpm workspace:outdated

workspace-update: ## Update all dependencies
	@echo "$(BLUE)Updating dependencies...$(RESET)"
	pnpm update --recursive

# ===========================================
# Legacy Commands (for compatibility)
# ===========================================

.PHONY: build-local force-build-frontend push-frontend build-backend push-backend

build-local: build ## Legacy: Build local
build-frontend: docker-build-frontend ## Legacy: Build frontend
push-frontend: docker-build-frontend ## Legacy: Push frontend
build-backend: docker-build-backend ## Legacy: Build backend
push-backend: docker-build-backend ## Legacy: Push backend

build-backend: build-local
	@echo $(TAG) > $(TAG_FILE)
	docker buildx build \
		--platform linux/amd64 \
		-t $(BACKEND_IMAGE):$(ENV)-$(TAG) \
		-t $(BACKEND_IMAGE):$(ENV)-latest \
		-f apps/backend/Dockerfile \
		--build-context backend=. \
		--push .

push-backend:
	docker push $(BACKEND_IMAGE):$(ENV)-$(TAG)
	docker push $(BACKEND_IMAGE):$(ENV)-latest

# Combined commands
build: build-frontend build-backend
push: push-frontend push-backend

# Help command
help:
	@echo "Available commands:"
	@echo "\nLocal Development:"
	@echo "  install            : Install all dependencies using bun"
	@echo "  dev               : Start development servers"
	@echo "  build-local       : Build all packages locally"
	@echo "  clean             : Clean all build caches and node_modules"
	@echo "\nDocker Commands:"
	@echo "  build-frontend    : Build frontend Docker image (ENV=development|production)"
	@echo "  build-backend     : Build backend Docker image (ENV=development|production)"
	@echo "  force-build-frontend : Build frontend Docker image without cache (ENV=development|production)"
	@echo "  push-frontend     : Push frontend Docker image to registry (ENV=development|production)"
	@echo "  push-backend      : Push backend Docker image to registry (ENV=development|production)"
	@echo "  build            : Build all images (ENV=development|production)"
	@echo "  push             : Push all images to registry (ENV=development|production)"
	@echo
	@echo "Usage examples:"
	@echo "  make install                        : Install dependencies"
	@echo "  make dev                           : Start development environment"
	@echo "  make build ENV=development         : Build development images"
	@echo "  make push ENV=production          : Push production images"

.DEFAULT_GOAL := help

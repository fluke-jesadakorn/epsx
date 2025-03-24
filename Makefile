# Docker registry and project variables
REGISTRY := us-central1-docker.pkg.dev/epsx-449804/epsx
PROJECT_NAME := epsx

# Environment (default to development)
ENV ?= development

# Image names
FRONTEND_IMAGE := $(REGISTRY)/frontend
BACKEND_IMAGE := $(REGISTRY)/backend

# Tag with timestamp
TAG_FILE := .docker-tag
TAG := $(shell cat $(TAG_FILE) 2>/dev/null || date +%Y%m%d-%H%M%S)

# Docker build platforms
PLATFORMS := linux/amd64

# Bun commands
.PHONY: install build-local dev clean

install:
	bun install

dev:
	bun turbo run dev --parallel

build-local:
	bun turbo run build

clean:
	bun turbo run clean
	rm -rf node_modules
	rm -rf .turbo

# Docker commands
.PHONY: build-frontend force-build-frontend push-frontend build-backend push-backend build push docker-build docker-push help

build-frontend: build-local
	@echo $(TAG) > $(TAG_FILE)
	docker build \
		--platform linux/amd64 \
		-t $(FRONTEND_IMAGE):$(ENV)-$(TAG) \
		-t $(FRONTEND_IMAGE):$(ENV)-latest \
		-f apps/frontend/Dockerfile \
		--build-context frontend=. \
		--push .

push-frontend: build-frontend

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

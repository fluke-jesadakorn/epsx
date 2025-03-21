# Docker registry and project variables
REGISTRY := us-central1-docker.pkg.dev/epsx-449804/epsx
PROJECT_NAME := epsx

# Environment (default to development)
ENV ?= development

# Image names
FRONTEND_IMAGE := $(REGISTRY)/frontend

# Tag with timestamp
TAG_FILE := .docker-tag
TAG := $(shell cat $(TAG_FILE) 2>/dev/null || date +%Y%m%d-%H%M%S)

# Docker build platforms
PLATFORMS := linux/amd64

.PHONY: build-frontend force-build-frontend push-frontend build push help

# Frontend commands
build-frontend:
	@echo $(TAG) > $(TAG_FILE)
	docker buildx build \
		--platform $(PLATFORMS) \
		-t $(FRONTEND_IMAGE):$(ENV)-$(TAG) \
		-t $(FRONTEND_IMAGE):$(ENV)-latest \
		-f Dockerfile \
		--build-context frontend=. .

force-build-frontend:
	@echo $(TAG) > $(TAG_FILE)
	docker build \
		--no-cache \
		--platform $(PLATFORMS) \
		-t $(FRONTEND_IMAGE):$(ENV)-$(TAG) \
		-t $(FRONTEND_IMAGE):$(ENV)-latest \
		--progress=plain \
		-f Dockerfile \
		--build-context frontend=. .

push-frontend:
	docker push $(FRONTEND_IMAGE):$(ENV)-$(TAG)
	docker push $(FRONTEND_IMAGE):$(ENV)-latest

# Combined commands
build: build-frontend
push: push-frontend

# Help command
help:
	@echo "Available commands:"
	@echo "  build-frontend       : Build frontend Docker image (ENV=development|production)"
	@echo "  force-build-frontend : Build frontend Docker image without cache (ENV=development|production)"
	@echo "  push-frontend       : Push frontend Docker image to registry (ENV=development|production)"
	@echo "  build              : Build all images (ENV=development|production)"
	@echo "  push               : Push all images to registry (ENV=development|production)"
	@echo
	@echo "Usage examples:"
	@echo "  make build ENV=development             : Build development images"
	@echo "  make force-build-frontend ENV=production : Build production frontend image without cache"
	@echo "  make push ENV=production              : Push production images"

.DEFAULT_GOAL := help

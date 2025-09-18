# EPSX Development Container with Production Domain Support
# Optimized development environment with all tools pre-configured

FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# Set up development user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Install system dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    # Build tools
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    # Network tools for domain development
    curl \
    wget \
    net-tools \
    dnsutils \
    iputils-ping \
    # Development utilities
    git \
    vim \
    nano \
    htop \
    tree \
    jq \
    unzip \
    # SSL tools for certificate handling
    openssl \
    ca-certificates \
    # Database client tools
    postgresql-client \
    redis-tools \
    && apt-get autoremove -y && apt-get clean -y

# Install Node.js (handled by dev container features)
# Install Rust (handled by dev container features)

# Note: pnpm, diesel, and cargo tools will be installed via postCreateCommand
# after dev container features are applied

# Create workspace directory
RUN mkdir -p /workspace && chown ${USERNAME}:${USERNAME} /workspace

# Set up shell environment for development
RUN echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> /home/${USERNAME}/.bashrc \
    && echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> /home/${USERNAME}/.zshrc

# Configure Git to be safe with the workspace
RUN git config --system --add safe.directory /workspace

# Install development certificates for local HTTPS
RUN mkdir -p /usr/local/share/ca-certificates/epsx-dev

# Set working directory
WORKDIR /workspace

# Switch to development user
USER ${USERNAME}

# Set environment variables for development
ENV NODE_ENV=development
ENV RUST_ENV=development
ENV ENV=development
ENV PATH="/home/${USERNAME}/.cargo/bin:${PATH}"

# Expose common development ports
EXPOSE 3000 3001 8080 5432 6379
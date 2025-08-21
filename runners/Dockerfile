# GitHub Actions Self-Hosted Runner for Aurelia
FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
ENV RUNNER_ALLOW_RUNASROOT=1

# Install base dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    jq \
    build-essential \
    libssl-dev \
    libffi-dev \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev \
    zip \
    unzip \
    sudo \
    ca-certificates \
    gnupg \
    lsb-release \
    pkg-config \
    openssh-client \
    && rm -rf /var/lib/apt/lists/*

# Install Docker CLI (for Docker-in-Docker operations if needed)
RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && \
    apt-get install -y docker-ce-cli && \
    rm -rf /var/lib/apt/lists/*

# Install Rust
ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust
ENV PATH=/opt/rust/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    rustup component add rustfmt clippy && \
    rustup target add x86_64-unknown-linux-gnu

# Install cross for cross-compilation
RUN cargo install cross --version 0.2.5

# Install additional tools for Aurelia
RUN pip3 install --no-cache-dir \
    requests \
    paramiko \
    pyyaml \
    python-dotenv

# Create runner user (optional, but recommended for security)
RUN useradd -m -s /bin/bash runner && \
    usermod -aG sudo runner && \
    echo "runner ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Create work directory
WORKDIR /home/runner

# Download and configure GitHub Actions runner (latest stable version)
ARG RUNNER_VERSION=2.328.0
ENV RUNNER_VERSION=${RUNNER_VERSION}

# Download runner with hash verification (as per GitHub's official guide)
RUN curl -o actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz -L \
    https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    echo "01066fad3a2893e63e6ca80ae3a1fad5bf9329d60e77ee15f2b97c148c3cd4e  actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" | shasum -a 256 -c && \
    tar xzf ./actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    rm actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz && \
    ./bin/installdependencies.sh && \
    # Prevent auto-update issues by setting correct permissions
    chmod +x ./bin/Runner.Listener ./bin/Runner.Worker ./bin/Runner.PluginHost ./run.sh ./run-helper.sh

# Copy startup script
COPY entrypoint.sh /home/runner/entrypoint.sh
RUN chmod +x /home/runner/entrypoint.sh

# Set up cargo cache directory
ENV CARGO_HOME=/home/runner/.cargo
RUN mkdir -p /home/runner/.cargo && \
    echo '[net]' > /home/runner/.cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /home/runner/.cargo/config.toml

# Pre-cache common Rust dependencies for Aurelia (optional, can be removed to speed up build)
# RUN cargo install cargo-cache cargo-audit cargo-outdated

# Clean up
RUN apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Labels for organization
LABEL maintainer="Aurelia Team"
LABEL description="GitHub Actions self-hosted runner for Aurelia project"
LABEL version="1.0.0"

ENTRYPOINT ["/home/runner/entrypoint.sh"]
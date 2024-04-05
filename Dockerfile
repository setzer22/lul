FROM ubuntu:18.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    gcc \
    git \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
    
# Setup rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add cargo to PATH
ENV PATH="/root/.cargo/bin:${PATH}"
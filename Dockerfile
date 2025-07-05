# ===========================
# Stage 1: Builder
# ===========================
FROM rust:1.86-slim as builder

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        libsqlite3-dev \
        build-essential \
        curl \
        tar \
        && rm -rf /var/lib/apt/lists/*

# Install Binaryen from source
ENV BINARYEN_VERSION=120_b
ENV BINARYEN_URL=https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz

RUN curl -L -o binaryen.tar.gz ${BINARYEN_URL} && \
    tar -xzf binaryen.tar.gz && \
    mv binaryen-version_${BINARYEN_VERSION} /opt/binaryen && \
    ln -s /opt/binaryen/bin/wasm-opt /usr/local/bin/wasm-opt && \
    rm binaryen.tar.gz

# Rust targets & Trunk for frontend
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

# ---------- workspace --------------------------------------------------------
WORKDIR /app

# Copy Cargo.toml and lock files
COPY Cargo.toml Cargo.lock ./
COPY frontend_simple_web/Cargo.toml frontend_simple_web/Cargo.toml
COPY backend_simple_web/Cargo.toml backend_simple_web/Cargo.toml

# Copy sources
COPY frontend_simple_web frontend_simple_web
COPY backend_simple_web  backend_simple_web

RUN cargo fetch

# ---------- build frontend (Yew/Trunk) --------------------------------------
WORKDIR /app/frontend_simple_web
RUN trunk build --release --dist=dist

# ---------- build backend ----------------------------------------------------
WORKDIR /app/backend_simple_web
RUN cargo build --release

# ===========================
# Stage 2: Runtime Image
# ===========================
FROM nginx:stable

# Install necessary runtime dependencies and Rust toolchain
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        openssl \
        pkg-config \
        libssl-dev \
        build-essential \
        curl \
        libsqlite3-dev \
        && rm -rf /var/lib/apt/lists/*

# Install Rust using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Update PATH to include Cargo's bin directory
ENV PATH="/root/.cargo/bin:${PATH}"

# Replace the default Nginx root with our built frontend
RUN rm -rf /usr/share/nginx/html/*
COPY --from=builder /app/frontend_simple_web/dist /usr/share/nginx/html

# Copy the Nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy the compiled backend binary
COPY --from=builder /app/target/release/backend_simple_web /usr/local/bin/backend_simple_web

# Create the public_site directory
RUN mkdir -p /public_site

# Copy the run script
COPY run.sh /run.sh
RUN chmod +x /run.sh

# Expose necessary ports
EXPOSE 8000
EXPOSE 8080

# Set the entrypoint to the run script
CMD ["/run.sh"]

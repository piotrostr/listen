FROM rust:1.79 AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install latest protoc
RUN PROTOC_VERSION="29.2" && \
    curl -LO "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip" && \
    unzip "protoc-${PROTOC_VERSION}-linux-x86_64.zip" -d /usr/local && \
    rm "protoc-${PROTOC_VERSION}-linux-x86_64.zip"


WORKDIR /listen

# Copy only the files needed for dependency resolution
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies - this will be cached if dependencies don't change
RUN cargo build --release --locked

# Remove the dummy source
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN touch src/main.rs && \
    cargo build --release --locked

FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /listen/target/release/listen .

CMD ["./listen", "listen-service"]

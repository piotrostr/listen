FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --features full --release --recipe-path recipe.json
COPY . .
RUN cargo build --features full --release --bin kit

# main listen-kit port
EXPOSE 6969

FROM debian:bookworm-slim AS runtime
WORKDIR /app
# ensure certs
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kit /usr/local/bin
ENTRYPOINT ["/usr/local/bin/kit"]

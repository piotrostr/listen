FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --features http --features solana --release --recipe-path recipe.json
COPY . .
RUN cargo build --features http --features solana --release --bin kit

# main listen-kit port
EXPOSE 6969

# engine ports
EXPOSE 6966
# internal, shouldn't have ingress
EXPOSE 6901 

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/kit /usr/local/bin
ENTRYPOINT ["/usr/local/bin/kit"]

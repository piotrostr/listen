FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --features http --features solana --release --recipe-path recipe.json
COPY . .
RUN cargo build --features http --features solana --release --bin server

EXPOSE 6969

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server"]

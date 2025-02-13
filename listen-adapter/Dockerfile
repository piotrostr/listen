FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

EXPOSE 6969

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
	ca-certificates \
	openssl \
	libssl3 \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/adapter /usr/local/bin
ENTRYPOINT ["/usr/local/bin/adapter"]

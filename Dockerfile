FROM ubuntu:22.04
COPY ./target/release/listen /usr/local/bin/listen

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

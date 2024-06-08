#!/bin/bash

git pull && \
  cargo build --release && \
    docker build -t piotrostr/listen . && \
    docker push piotrostr/listen

#!/bin/bash

cargo run -- listen \
    --worker-count 10 \
    --buffer-size 1000


#!/bin/bash

sudo dtrace -c './target/release/listen --keypair-path /Users/piotrostr/.config/solana/id.json swap --input-mint sol --output-mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v --amount 1000000 -y' -o out.stacks -n 'profile-997 /execname == "listen"/ { @[ustack()] = count(); }'

stackcollapse.pl out.stacks | flamegraph.pl > flamegraph.svg

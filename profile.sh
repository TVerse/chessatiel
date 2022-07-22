#!/usr/bin/env bash

cargo build --release && \
  perf record -g -F 999 --call-graph dwarf ./target/release/chessatiel --profile-mode "$PROFILE_MODE" && \
  perf script -F +pid > "$PROFILE_MODE.perf"

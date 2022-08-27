#!/usr/bin/env bash

if [ -z "$PROFILE_MODE" ]
then
  echo "PROFILE_MODE not set"
  exit 1
fi

cargo build --profile perf && \
  perf record -g -F 999 --call-graph dwarf ./target/perf/chessatiel --profile-mode "$PROFILE_MODE" && \
  perf script -F +pid > "$PROFILE_MODE.perf"

#!/bin/bash

# Tail cargo output with colors
RUST_LOG=debug cargo run 2>&1 | grep -E "INFO|WARN|ERROR|DEBUG" --color=always
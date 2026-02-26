#!/bin/bash
# Onevox Daemon Runner
# Simple wrapper to run the daemon with proper logging

# Run the daemon with all arguments passed through
exec "$(dirname "$0")/../target/release/onevox" daemon "$@"

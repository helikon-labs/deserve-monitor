#!/usr/bin/env bash

set -euo pipefail

cargo +nightly fmt
cargo +nightly clippy --all-targets -- -D warnings -W clippy::too_many_lines -W clippy::excessive_nesting
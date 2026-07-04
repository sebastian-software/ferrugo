#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

node scripts/generate_readme_benchmark_results.mjs --check

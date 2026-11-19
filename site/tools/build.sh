#!/usr/bin/env bash
# Assembles the deployable site in <out-dir> (default: _site):
# static files from site/ plus data generated from the real novastack binary.
#
#   site/tools/build.sh [<out-dir>] [<novastack-binary>]
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:-$root/_site}"
bin="${2:-}"

if [[ -z "$bin" ]]; then
  cargo build --release --locked --manifest-path "$root/Cargo.toml"
  bin="$(cargo metadata --format-version 1 --no-deps --manifest-path "$root/Cargo.toml" \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/release/novastack"
fi

mkdir -p "$out"
rsync -a --delete --exclude 'tools/' --exclude '.gitignore' --exclude 'data/generated/' "$root/site/" "$out/"
python3 -I "$root/site/tools/build-data.py" "$bin" "$out/data/generated"
echo "site assembled in $out"

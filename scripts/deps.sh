#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

mode="${1:-check}"
case "$mode" in
check | update) ;;
*) echo "usage: $0 check|update" >&2; exit 2 ;;
esac

age=$(mise settings get minimum_release_age)
[[ "$age" =~ ^[0-9]+d$ ]] || { echo "minimum_release_age must be in days, got '$age'" >&2; exit 2; }
days=${age%d}
[ "$days" -le 7 ] || { echo "minimum_release_age is $age; at most 7d is allowed" >&2; exit 2; }
cutoff_epoch=$(( $(date -u +%s) - days * 86400 ))
cutoff=$(date -u -d "@$cutoff_epoch" +%Y-%m-%dT%H:%M:%S 2>/dev/null || date -u -r "$cutoff_epoch" +%Y-%m-%dT%H:%M:%S)
user_agent="User-Agent: diffmate-deps (https://github.com/paderinandrey/diffmate)"

semver() { [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "unexpected version '$1'" >&2; exit 2; }; }
newer() { [ "$1" != "$2" ] && [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1)" = "$2" ]; }

rust_eligible=$(gh api 'repos/rust-lang/rust/releases?per_page=50' \
  --jq '.[] | select((.draft or .prerelease) | not) | [.tag_name, .published_at] | @tsv' |
  awk -F'\t' -v c="$cutoff" '$1 ~ /^[0-9]+\.[0-9]+\.[0-9]+$/ && substr($2, 1, 19) <= c { print $1 }' |
  sort -V | tail -1)
[ -n "$rust_eligible" ] || { echo "no eligible Rust release found" >&2; exit 2; }
rust_channel=$(sed -nE 's/^channel = "(.*)"$/\1/p' rust-toolchain.toml)
rust_version=$(sed -nE 's/^rust-version = "(.*)"$/\1/p' Cargo.toml)
semver "$rust_eligible"; semver "$rust_channel"; semver "$rust_version"

tools=()
while IFS= read -r tool; do tools+=("$tool"); done < <(
  awk '/^\[/ { in_tools = ($0 == "[tools]"); next } in_tools && /=/ { sub(/ *=.*/, ""); gsub(/"/, ""); print }' mise.toml)
hk_pin=$(sed -nE 's/^hk = "(.*)"$/\1/p' mise.toml)
hk_pkl=$(sed -nE 's#.*/releases/download/v([0-9.]+)/hk@.*#\1#p' hk.pkl)
semver "$hk_pin"; semver "$hk_pkl"

tool_rows=$(mise outdated --bump --json "${tools[@]}" 2>/dev/null |
  jq -r 'to_entries[] | select(.value.bump != null and .value.bump != .value.requested) | "\(.key) \(.value.requested) -> \(.value.bump)"')

crates=$(cargo metadata --locked --format-version 1 | jq -r '. as $m | $m.resolve.nodes[] | select(.id == $m.resolve.root) | .deps[]
  | select(.pkg | startswith("registry+")) | [.dep_kinds[].kind] as $k | (.pkg | split("#")[1] | split("@")) as $p
  | "\($p[0]) \($p[1]) \(if ($k | index(null)) != null then "normal" elif ($k | index("build")) != null then "build" else "dev" end)"')
behind_crates=""
while read -r name current kind; do
  [ -n "$name" ] || continue
  [[ "$name" =~ ^[A-Za-z0-9_-]+$ ]] || { echo "unexpected crate name '$name'" >&2; exit 2; }
  semver "$current"
  sleep 1
  eligible=$(curl -fsS -H "$user_agent" "https://crates.io/api/v1/crates/$name/versions?per_page=100" |
    jq -r --arg c "$cutoff" '.versions[] | select((.yanked | not) and (.num | test("^[0-9]+\\.[0-9]+\\.[0-9]+$")) and (.created_at[0:19] <= $c)) | .num' |
    sort -V | tail -1)
  [ -n "$eligible" ] || continue
  semver "$eligible"
  if newer "$current" "$eligible"; then behind_crates+="$name $kind $current $eligible"$'\n'; fi
done <<< "$crates"

if [ "$mode" = check ]; then
  behind=0
  report() { echo "outdated: $1"; behind=1; }
  newer "$rust_channel" "$rust_eligible" && report "rust-toolchain.toml channel $rust_channel -> $rust_eligible"
  newer "$rust_version" "$rust_eligible" && report "Cargo.toml rust-version $rust_version -> $rust_eligible"
  [ "$hk_pin" = "$hk_pkl" ] || report "hk.pkl amends hk $hk_pkl -> $hk_pin"
  while IFS= read -r row; do [ -n "$row" ] || continue; report "mise tool $row"; done <<< "$tool_rows"
  while read -r name kind current eligible; do [ -n "$name" ] || continue; report "crate $name ($kind) $current -> $eligible"; done <<< "$behind_crates"
  if [ "$behind" = 1 ]; then echo "run: mise run deps:update" >&2; exit 1; fi
  echo "Rust $rust_eligible, mise tools and direct crates are on their latest eligible releases"
  exit 0
fi

newer "$rust_channel" "$rust_eligible" && sed -i.bak -E "s/^channel = \".*\"/channel = \"$rust_eligible\"/" rust-toolchain.toml
newer "$rust_version" "$rust_eligible" && sed -i.bak -E "s/^rust-version = \".*\"/rust-version = \"$rust_eligible\"/" Cargo.toml
rm -f rust-toolchain.toml.bak Cargo.toml.bak
mise upgrade --bump "${tools[@]}"
hk_new=$(sed -nE 's/^hk = "(.*)"$/\1/p' mise.toml)
semver "$hk_new"
sed -i.bak -E "s#/download/v[0-9.]+/hk@[0-9.]+#/download/v$hk_new/hk@$hk_new#" hk.pkl
rm -f hk.pkl.bak
while read -r name kind _ eligible; do
  [ -n "$name" ] || continue
  flag=""
  case "$kind" in dev) flag="--dev" ;; build) flag="--build" ;; esac
  cargo add ${flag:+"$flag"} "$name@$eligible"
  cargo update -p "$name" --precise "$eligible"
done <<< "$behind_crates"
echo "updated Rust, mise tools and direct crates; run: mise run lint && mise run test"

#!/bin/sh
S='@STATE@'
n=1
while ! mkdir "$S/calls/$n" 2>/dev/null; do n=$((n+1)); done
for a in "$@"; do printf '%s\n' "$a"; done > "$S/calls/$n/argv"
for r in "$S"/routes/*.route; do
  [ -e "$r" ] || continue
  prefix=$(head -n 1 "$r")
  case "$*" in
    "$prefix"*)
      tail -n +2 "$r"
      if [ -f "${r%.route}.stderr" ]; then cat "${r%.route}.stderr" >&2; fi
      if [ -f "${r%.route}.exit" ]; then exit "$(cat "${r%.route}.exit")"; fi
      exit 0 ;;
  esac
done
echo "fake gh: no route for: $*" >&2
exit 1

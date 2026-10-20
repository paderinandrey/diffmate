#!/bin/sh
S='@STATE@'
n=1
while ! mkdir "$S/calls/$n" 2>/dev/null; do n=$((n+1)); done
d="$S/calls/$n"
for a in "$@"; do printf '%s\n' "$a"; done > "$d/argv"
env | sort > "$d/env"
pwd > "$d/cwd"
echo $$ > "$d/pid"
if [ "$1" = "--version" ]; then cat "$S/version"; exit 0; fi
cat > "$d/stdin"
if [ ! -s "$d/stdin" ]; then
  cat "$S/probe_stderr" >&2
  exit "$(cat "$S/probe_exit")"
fi
if [ -e "$S/snapshot_tree" ]; then cp -R . "$d/tree"; fi
r=$(( $(cat "$S/run_count" 2>/dev/null || echo 0) + 1 ))
echo "$r" > "$S/run_count"
prev=
for a in "$@"; do
  if [ "$prev" = "--add-dir" ]; then cp -R "$a" "$d/context"; fi
  prev=$a
done
if [ -e "$S/spawn_child" ]; then
  sleep 300 </dev/null >/dev/null 2>&1 &
  echo $! > "$d/child.pid"
fi
if [ -e "$S/hold_run" ] && [ "$(cat "$S/hold_run")" = "$r" ]; then
  echo started > "$S/started"
  read -r _ < "$S/release"
fi
cat "$S/run_stdout"
exit "$(cat "$S/run_exit")"

#!/usr/bin/env bash
set -u
expected=$(printf 'alpha\nbeta\ngamma\ndelta\n')
actual=$(cat notes.log 2>/dev/null || true)
if [[ "$actual" == "$expected" ]]; then
  echo "notes.log has all four lines in the right order."
  exit 0
fi
echo "notes.log does not match. Got:"
echo "$actual"
exit 1

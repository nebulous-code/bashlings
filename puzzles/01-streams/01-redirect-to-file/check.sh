#!/usr/bin/env bash
set -u
if [[ ! -f out.txt ]]; then
  echo "out.txt does not exist."
  exit 1
fi
expected=$(printf 'alpha\nbeta\ngamma\n')
actual=$(cat out.txt)
if [[ "$actual" == "$expected" ]]; then
  echo "out.txt contains the listing of /puzzle/data."
  exit 0
fi
echo "out.txt contents do not match the expected listing."
echo "got:"
echo "$actual"
exit 1

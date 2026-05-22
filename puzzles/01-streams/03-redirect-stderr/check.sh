#!/usr/bin/env bash
set -u
if [[ ! -f clean.txt ]]; then
  echo "clean.txt does not exist."
  exit 1
fi
expected=$(printf 'useful line 1\nuseful line 2\nuseful line 3\n')
actual=$(cat clean.txt)
if [[ "$actual" == "$expected" ]]; then
  echo "clean.txt has the useful lines and no stderr noise."
  exit 0
fi
echo "clean.txt contents are not what we wanted. Got:"
echo "$actual"
exit 1

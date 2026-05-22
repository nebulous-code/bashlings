#!/usr/bin/env bash
set -u
if [[ ! -f count.txt ]]; then
  echo "count.txt does not exist."
  exit 1
fi
actual=$(cat count.txt)
if [[ "$actual" == "17" ]]; then
  echo "count.txt holds the correct line count."
  exit 0
fi
echo "count.txt should contain just the number 17. Got: '$actual'"
exit 1

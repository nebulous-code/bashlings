#!/usr/bin/env bash
set -u
if [[ ! -f c.txt ]]; then
  echo "c.txt does not exist."
  exit 1
fi
expected=$({ tail -n 20 a.txt; head -n 5 b.txt; })
actual=$(cat c.txt)
if [[ "$actual" == "$expected" ]]; then
  echo "c.txt contains the right 25 lines in the right order."
  exit 0
fi
echo "c.txt contents do not match."
diff <(echo "$expected") <(echo "$actual") || true
exit 1

#!/usr/bin/env bash
set -euo pipefail
# 50 lines, "a-001" .. "a-050"
seq -f 'a-%03g' 1 50 > a.txt
# 30 lines, "b-001" .. "b-030"
seq -f 'b-%03g' 1 30 > b.txt
rm -f c.txt

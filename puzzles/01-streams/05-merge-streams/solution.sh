#!/usr/bin/env bash
set -euo pipefail
{ tail -n 20 a.txt; head -n 5 b.txt; } > c.txt

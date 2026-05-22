#!/usr/bin/env bash
set -euo pipefail
cat > noisy.sh <<'EOF'
#!/usr/bin/env bash
echo "useful line 1"
echo "warning: legacy mode" >&2
echo "useful line 2"
echo "warning: deprecated flag" >&2
echo "useful line 3"
EOF
chmod +x noisy.sh
rm -f clean.txt

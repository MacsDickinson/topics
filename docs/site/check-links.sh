#!/usr/bin/env bash
# Fails if a relative src/href in any HTML file under the given directory points at a missing file.
set -euo pipefail
root="${1:?usage: check-links.sh <site-dir>}"
status=0
while IFS= read -r -d '' html; do
  dir=$(dirname "$html")
  while IFS= read -r ref; do
    case "$ref" in
      ''|'#'*|http://*|https://*|//*|mailto:*|data:*|javascript:*) continue ;;
      /*) echo "ERROR: $html: absolute path '$ref' breaks Pages sub-paths"; status=1; continue ;;
    esac
    path=${ref%%[#?]*}
    [ -z "$path" ] && continue
    if [ ! -e "$dir/$path" ]; then
      echo "ERROR: $html: '$ref' not found"; status=1
    fi
  done < <(grep -oE '(src|href)="[^"]*"' "$html" | sed -E 's/^(src|href)="//; s/"$//' || true)
done < <(find "$root" -name '*.html' -print0)
[ "$status" -eq 0 ] && echo "Link check passed for $root"
exit "$status"

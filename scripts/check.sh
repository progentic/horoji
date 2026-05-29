#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(pwd)}"

python3 "$ROOT/scripts/validate_governance.py" --root "$ROOT"

# Validate all schema files are parseable JSON. Full Draft 2020-12 validation is
# intentionally left to CI environments that install a dedicated schema tool.
python3 - <<'PY' "$ROOT"
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
for path in sorted((root / "json").glob("*.schema.json")):
    with path.open("r", encoding="utf-8") as handle:
        json.load(handle)
    print(f"valid json: {path.relative_to(root)}")
PY

echo "mechanical consistency checks passed"

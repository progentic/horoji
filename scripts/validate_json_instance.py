#!/usr/bin/env python3
"""Small JSON instance validator for the governance pack.

This is a deliberately small validator for CI smoke checks. It supports the subset
of JSON Schema used by this repository's schemas: type, required, properties,
additionalProperties=false, enum, const, minLength, minItems, maxItems, pattern,
items, and local $defs/$ref. Use a full JSON Schema implementation for complete
Draft 2020-12 coverage.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


class ValidationError(Exception):
    pass


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def resolve_ref(schema: dict[str, Any], ref: str) -> dict[str, Any]:
    if not ref.startswith("#/"):
        raise ValidationError(f"external ref not supported by this smoke validator: {ref}")
    node: Any = schema
    for part in ref[2:].split("/"):
        node = node[part]
    if not isinstance(node, dict):
        raise ValidationError(f"ref does not resolve to schema object: {ref}")
    return node


def check_type(value: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return (isinstance(value, int) or isinstance(value, float)) and not isinstance(value, bool)
    if expected == "null":
        return value is None
    return True


def validate_node(root_schema: dict[str, Any], node_schema: dict[str, Any], value: Any, path: str) -> list[str]:
    errors: list[str] = []

    if "$ref" in node_schema:
        try:
            node_schema = resolve_ref(root_schema, node_schema["$ref"])
        except ValidationError as exc:
            return [f"{path}: {exc}"]

    if "const" in node_schema and value != node_schema["const"]:
        errors.append(f"{path}: expected const {node_schema['const']!r}")

    if "enum" in node_schema and value not in node_schema["enum"]:
        errors.append(f"{path}: expected one of {node_schema['enum']!r}")

    expected_type = node_schema.get("type")
    if expected_type and not check_type(value, expected_type):
        errors.append(f"{path}: expected type {expected_type}")
        return errors

    if isinstance(value, str):
        if "minLength" in node_schema and len(value) < int(node_schema["minLength"]):
            errors.append(f"{path}: string shorter than minLength {node_schema['minLength']}")
        if "pattern" in node_schema and not re.search(node_schema["pattern"], value):
            errors.append(f"{path}: string does not match pattern {node_schema['pattern']!r}")

    if isinstance(value, list):
        if "minItems" in node_schema and len(value) < int(node_schema["minItems"]):
            errors.append(f"{path}: array shorter than minItems {node_schema['minItems']}")
        if "maxItems" in node_schema and len(value) > int(node_schema["maxItems"]):
            errors.append(f"{path}: array longer than maxItems {node_schema['maxItems']}")
        if node_schema.get("uniqueItems"):
            seen = set()
            for item in value:
                marker = json.dumps(item, sort_keys=True)
                if marker in seen:
                    errors.append(f"{path}: array items must be unique")
                    break
                seen.add(marker)
        item_schema = node_schema.get("items")
        if isinstance(item_schema, dict):
            for idx, item in enumerate(value):
                errors.extend(validate_node(root_schema, item_schema, item, f"{path}[{idx}]"))

    if isinstance(value, dict):
        required = node_schema.get("required", [])
        for key in required:
            if key not in value:
                errors.append(f"{path}: missing required key {key!r}")

        properties = node_schema.get("properties", {})
        if node_schema.get("additionalProperties") is False:
            for key in value:
                if key not in properties:
                    errors.append(f"{path}: additional property not allowed: {key!r}")

        for key, sub_schema in properties.items():
            if key in value and isinstance(sub_schema, dict):
                errors.extend(validate_node(root_schema, sub_schema, value[key], f"{path}.{key}"))

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate a JSON instance against a repository schema subset.")
    parser.add_argument("schema", help="path to JSON schema")
    parser.add_argument("instance", help="path to JSON instance")
    args = parser.parse_args()

    schema = load_json(Path(args.schema))
    instance = load_json(Path(args.instance))
    errors = validate_node(schema, schema, instance, "$")

    if errors:
        print("JSON instance validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("JSON instance validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

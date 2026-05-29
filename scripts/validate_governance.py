#!/usr/bin/env python3
"""Validate mechanical consistency for the acceptability engineering governance pack.

This script intentionally uses only the Python standard library. It does not fully
implement JSON Schema. It validates that schema files are well-formed and that the
Markdown governance files preserve the required authority model, phase map, and
cross-document links.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Iterable

REQUIRED_DOCS = [
    "docs/AGENTS.md",
    "docs/ARCHITECTURE.md",
    "docs/CODING_STYLE.md",
    "docs/GOVERNANCE.md",
    "docs/INVARIANTS.md",
    "docs/PHASEMAP.md",
]

REQUIRED_SCHEMAS = [
    "json/bounded-change-contract.schema.json",
    "json/evidence-record.schema.json",
    "json/rejection-telemetry.schema.json",
    "json/counterexample.schema.json",
    "json/governance-pack.schema.json",
]

REQUIRED_SCRIPTS = [
    "scripts/check.sh",
    "scripts/validate_governance.py",
    "scripts/validate_json_instance.py",
]

CORE_INVARIANT_PHRASES = [
    "The LLM is non-authoritative",
    "The harness owns the authority boundary",
    "Generated code is untrusted by default",
    "Every admissible change carries evidence",
    "Human judgment revises the acceptability surface",
    "Rejected artifacts must not cross into trusted state",
    "State transitions must be explicit and auditable",
    "Evidence must be structured enough for replay",
    "Deployment approval must not be inferred from generation success",
]

REQUIRED_GOVERNANCE_PHRASES = [
    "The model is not authoritative",
    "The harness owns mechanical admissibility",
    "Humans own semantic judgment",
    "LLM: S x G -> C*",
    "B: C x A -> {0, 1}",
    "B(c, A) = 1 -> admissible",
    "B(c, A) = 0 -> rejected + telemetry T(c)",
]

FORBIDDEN_PHRASES = [
    "Complete Authority",
    "LLM owns authority",
    "model owns authority",
    "generated output is trusted by default",
    "deployment approval is inferred",
    "AI approval is sufficient",
    "human review is unnecessary",
]



REQUIRED_RUST_FILES = [
    "crates/horoji-core/Cargo.toml",
    "crates/horoji-core/src/lib.rs",
    "crates/horoji-core/src/contract.rs",
    "crates/horoji-core/src/evidence.rs",
]

AUTHORITY_RUST_MODULES = [
    "crates/horoji-core/src/contract.rs",
    "crates/horoji-core/src/evidence.rs",
]

REQUIRED_RUST_TEST_NAMES = [
    "contract_json_is_stable_across_repeated_serialization",
    "evidence_json_is_stable_when_gate_insertion_order_changes",
    "from_json_rejects_structurally_valid_but_policy_invalid_contract",
    "failed_gate_blocks_promotion",
]

FORBIDDEN_RUST_CORE_TERMS = [
    "HashMap",
    "HashSet",
    "std::fs",
    "std::net",
    "std::process",
    "std::env",
    "tokio::net",
    "reqwest",
    "Command::new",
]

PHASES = {
    "L0": "Ungoverned Generation",
    "L1": "Syntax and Schema Enforcement",
    "L2": "Harness Integration",
    "L3": "Semantic Loop",
    "L4": "Actively Constrained",
    "L5": "Mature Admissibility",
}


MAX_FILE_LOC = 1200
MAX_PY_FUNCTION_LOC = 180
MAX_PY_CLASS_LOC = 500
MAX_PY_BLOCK_NESTING = 5
MAX_JSON_DEPTH = 8
MAX_MARKDOWN_HEADING_DEPTH = 4
MAX_BRACE_NESTING = 6

TEXT_FILE_SUFFIXES = {
    ".md", ".py", ".sh", ".json", ".rs", ".toml", ".yml", ".yaml",
    ".ts", ".tsx", ".js", ".jsx", ".css", ".html", ".txt",
}

IGNORED_DIRS = {
    ".git", ".hg", ".svn", "target", "node_modules", "dist", "build", ".venv",
    "venv", "__pycache__", ".mypy_cache", ".pytest_cache", ".ruff_cache",
}


def read_text(root: Path, rel: str) -> str:
    path = root / rel
    return path.read_text(encoding="utf-8")


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def require_files(root: Path, paths: Iterable[str], errors: list[str]) -> None:
    for rel in paths:
        path = root / rel
        if not path.is_file():
            fail(errors, f"missing required file: {rel}")


def validate_schema_files(root: Path, errors: list[str]) -> None:
    for rel in REQUIRED_SCHEMAS:
        path = root / rel
        if not path.is_file():
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            fail(errors, f"invalid JSON in {rel}: {exc}")
            continue

        if data.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
            fail(errors, f"{rel} must declare JSON Schema draft 2020-12")
        if data.get("type") != "object":
            fail(errors, f"{rel} must use top-level type object")
        if "title" not in data:
            fail(errors, f"{rel} must include title")
        if "required" not in data:
            fail(errors, f"{rel} must include required fields")
        if "properties" not in data:
            fail(errors, f"{rel} must include properties")


def validate_markdown_headings(root: Path, errors: list[str]) -> None:
    expected = {
        "docs/AGENTS.md": "# AGENTS",
        "docs/ARCHITECTURE.md": "# ARCHITECTURE",
        "docs/CODING_STYLE.md": "# CODING_STYLE",
        "docs/GOVERNANCE.md": "# GOVERNANCE",
        "docs/INVARIANTS.md": "# INVARIANTS",
        "docs/PHASEMAP.md": "# PHASEMAP",
    }
    for rel, heading in expected.items():
        if not (root / rel).is_file():
            continue
        first = read_text(root, rel).splitlines()[0].strip()
        if first != heading:
            fail(errors, f"{rel} heading must be exactly {heading!r}; got {first!r}")


def validate_agents_navigation(root: Path, errors: list[str]) -> None:
    if not (root / "docs/AGENTS.md").is_file():
        return
    text = read_text(root, "docs/AGENTS.md")
    for rel in [p for p in REQUIRED_DOCS if p != "docs/AGENTS.md"]:
        target = "/" + rel
        if target not in text:
            fail(errors, f"docs/AGENTS.md must reference {target}")


def validate_governance_text(root: Path, errors: list[str]) -> None:
    if not (root / "docs/GOVERNANCE.md").is_file():
        return
    text = read_text(root, "docs/GOVERNANCE.md")
    for phrase in REQUIRED_GOVERNANCE_PHRASES:
        if phrase not in text:
            fail(errors, f"docs/GOVERNANCE.md missing required phrase: {phrase}")


def validate_invariants(root: Path, errors: list[str]) -> None:
    if not (root / "docs/INVARIANTS.md").is_file():
        return
    text = read_text(root, "docs/INVARIANTS.md")
    for phrase in CORE_INVARIANT_PHRASES:
        if phrase not in text:
            fail(errors, f"docs/INVARIANTS.md missing invariant phrase: {phrase}")


def validate_phasemap(root: Path, errors: list[str]) -> None:
    if not (root / "docs/PHASEMAP.md").is_file():
        return
    text = read_text(root, "docs/PHASEMAP.md")
    for level, title in PHASES.items():
        pattern = rf"^## {re.escape(level)} - {re.escape(title)}$"
        if not re.search(pattern, text, flags=re.MULTILINE):
            fail(errors, f"docs/PHASEMAP.md missing phase heading: ## {level} - {title}")
    if text.count("Minimum exit condition:") != 6:
        fail(errors, "docs/PHASEMAP.md must contain exactly six 'Minimum exit condition:' entries")
    if "L5 - Mature Admissibility" not in text:
        fail(errors, "docs/PHASEMAP.md must use 'L5 - Mature Admissibility'")


def validate_forbidden_phrases(root: Path, errors: list[str]) -> None:
    for rel in REQUIRED_DOCS:
        path = root / rel
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8").lower()
        for phrase in FORBIDDEN_PHRASES:
            if phrase.lower() in text:
                fail(errors, f"{rel} contains forbidden phrase: {phrase}")


def validate_cross_terms(root: Path, errors: list[str]) -> None:
    required_terms = {
        "docs/ARCHITECTURE.md": ["Bounded Change Contract", "Rust Harness Orchestrator", "structured rejection telemetry"],
        "docs/CODING_STYLE.md": ["Candidate", "Admissible", "Rejected", "Evidence"],
        "docs/PHASEMAP.md": ["counterexample", "replay"],
    }
    for rel, terms in required_terms.items():
        path = root / rel
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8").lower()
        for term in terms:
            if term.lower() not in text:
                fail(errors, f"{rel} missing required cross-term: {term}")




def iter_governed_files(root: Path) -> Iterable[Path]:
    for path in root.rglob("*"):
        if any(part in IGNORED_DIRS for part in path.relative_to(root).parts):
            continue
        if not path.is_file():
            continue
        if path.suffix.lower() in TEXT_FILE_SUFFIXES:
            yield path


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8", errors="ignore").splitlines())


def json_depth(value: object, current: int = 0) -> int:
    if isinstance(value, dict):
        if not value:
            return current + 1
        return max(json_depth(item, current + 1) for item in value.values())
    if isinstance(value, list):
        if not value:
            return current + 1
        return max(json_depth(item, current + 1) for item in value)
    return current


def brace_nesting_depth(text: str) -> int:
    depth = 0
    max_depth = 0
    in_string: str | None = None
    escaped = False
    for char in text:
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == in_string:
                in_string = None
            continue
        if char in {'"', "'"}:
            in_string = char
        elif char in "{([":
            depth += 1
            max_depth = max(max_depth, depth)
        elif char in "})]":
            depth = max(0, depth - 1)
    return max_depth


def python_block_nesting(node: object, depth: int = 0) -> int:
    import ast

    block_nodes = (
        ast.If, ast.For, ast.AsyncFor, ast.While, ast.With, ast.AsyncWith, ast.Try,
        ast.Match, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef,
    )
    if isinstance(node, ast.AST):
        next_depth = depth + 1 if isinstance(node, block_nodes) else depth
        child_depths = [python_block_nesting(child, next_depth) for child in ast.iter_child_nodes(node)]
        return max([next_depth, *child_depths]) if child_depths else next_depth
    return depth



def validate_file_length(rel: str, text: str, errors: list[str]) -> None:
    loc = len(text.splitlines())
    if loc > MAX_FILE_LOC:
        fail(errors, f"{rel} has {loc} LOC; maximum is {MAX_FILE_LOC}. Split or refactor this god file.")


def validate_markdown_shape(rel: str, text: str, errors: list[str]) -> None:
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line.startswith("#"):
            continue
        level = len(line) - len(line.lstrip("#"))
        if level > MAX_MARKDOWN_HEADING_DEPTH:
            fail(errors, f"{rel}:{line_number} heading depth {level}; maximum is {MAX_MARKDOWN_HEADING_DEPTH}.")


def validate_json_shape(rel: str, text: str, errors: list[str]) -> None:
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        return
    depth = json_depth(data)
    if depth > MAX_JSON_DEPTH:
        fail(errors, f"{rel} JSON nesting depth {depth}; maximum is {MAX_JSON_DEPTH}. Split schema/object definitions.")


def python_node_size(node: object) -> int | None:
    lineno = getattr(node, "lineno", None)
    end_lineno = getattr(node, "end_lineno", None)
    if lineno is None or end_lineno is None:
        return None
    return int(end_lineno) - int(lineno) + 1


def validate_python_function(rel: str, node: object, errors: list[str]) -> None:
    name = getattr(node, "name", "<anonymous>")
    lineno = getattr(node, "lineno", 0)
    size = python_node_size(node)
    if size is not None and size > MAX_PY_FUNCTION_LOC:
        fail(errors, f"{rel}:{lineno} function {name!r} has {size} LOC; maximum is {MAX_PY_FUNCTION_LOC}.")
    nesting = python_block_nesting(node) - 1
    if nesting > MAX_PY_BLOCK_NESTING:
        fail(errors, f"{rel}:{lineno} function {name!r} nesting depth {nesting}; maximum is {MAX_PY_BLOCK_NESTING}.")


def validate_python_class(rel: str, node: object, errors: list[str]) -> None:
    name = getattr(node, "name", "<anonymous>")
    lineno = getattr(node, "lineno", 0)
    size = python_node_size(node)
    if size is not None and size > MAX_PY_CLASS_LOC:
        fail(errors, f"{rel}:{lineno} class {name!r} has {size} LOC; maximum is {MAX_PY_CLASS_LOC}.")


def validate_python_shape(rel: str, text: str, errors: list[str]) -> None:
    import ast

    try:
        tree = ast.parse(text, filename=rel)
    except SyntaxError as exc:
        fail(errors, f"{rel} has invalid Python syntax: {exc}")
        return
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            validate_python_function(rel, node, errors)
        elif isinstance(node, ast.ClassDef):
            validate_python_class(rel, node, errors)


def rust_brace_nesting_depth(text: str) -> int:
    depth = 0
    max_depth = 0
    in_string: str | None = None
    escaped = False
    for char in text:
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == in_string:
                in_string = None
            continue
        if char in {'\"', "'"}:
            in_string = char
        elif char == "{":
            depth += 1
            max_depth = max(max_depth, depth)
        elif char == "}":
            depth = max(0, depth - 1)
    return max_depth


def validate_brace_shape(rel: str, text: str, errors: list[str]) -> None:
    depth = brace_nesting_depth(text)
    if depth > MAX_BRACE_NESTING:
        fail(errors, f"{rel} brace nesting depth {depth}; maximum is {MAX_BRACE_NESTING}. Extract smaller units.")


def validate_rust_brace_shape(rel: str, text: str, errors: list[str]) -> None:
    depth = rust_brace_nesting_depth(text)
    if depth > MAX_BRACE_NESTING:
        fail(errors, f"{rel} brace nesting depth {depth}; maximum is {MAX_BRACE_NESTING}. Extract smaller units.")


def validate_text_file_shape(path: Path, root: Path, errors: list[str]) -> None:
    rel = path.relative_to(root).as_posix()
    text = path.read_text(encoding="utf-8", errors="ignore")
    suffix = path.suffix.lower()
    validate_file_length(rel, text, errors)
    if suffix == ".md":
        validate_markdown_shape(rel, text, errors)
    elif suffix == ".json":
        validate_json_shape(rel, text, errors)
    elif suffix == ".py":
        validate_python_shape(rel, text, errors)
    elif suffix == ".rs":
        validate_rust_brace_shape(rel, rust_non_test_text(text), errors)
    elif suffix in {".ts", ".tsx", ".js", ".jsx"}:
        validate_brace_shape(rel, text, errors)


def validate_file_size_and_shape(root: Path, errors: list[str]) -> None:
    """Prevent governance, scripts, schemas, and source files from becoming god files."""
    for path in iter_governed_files(root):
        validate_text_file_shape(path, root, errors)


def validate_pack_manifest(root: Path, errors: list[str]) -> None:
    schema = root / "json/governance-pack.schema.json"
    if not schema.is_file():
        return
    manifest = {
        "required_docs": REQUIRED_DOCS,
        "required_schemas": REQUIRED_SCHEMAS,
        "required_scripts": REQUIRED_SCRIPTS,
        "phase_levels": list(PHASES.keys()),
        "core_invariants": CORE_INVARIANT_PHRASES,
    }
    generated = root / "json/governance-pack.generated.json"
    generated.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")



def rust_non_test_text(text: str) -> str:
    """Return the governed portion before test modules for simple drift scans."""
    return text.split("#[cfg(test)]", 1)[0]


def validate_rust_authority_surface(root: Path, errors: list[str]) -> None:
    require_files(root, REQUIRED_RUST_FILES, errors)
    for rel in AUTHORITY_RUST_MODULES:
        path = root / rel
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        governed = rust_non_test_text(text)
        for term in FORBIDDEN_RUST_CORE_TERMS:
            if term in governed:
                fail(errors, f"{rel} uses forbidden governed-core term: {term}")
        if "unwrap()" in governed or ".unwrap()" in governed:
            fail(errors, f"{rel} uses unwrap() in governed code")
        if "expect(" in governed or ".expect(" in governed:
            fail(errors, f"{rel} uses expect() in governed code")
        public_id = re.compile(r"pub\s+struct\s+\w*Id\s*\(\s*(?:pub\s+String|pub\s*\(\s*crate\s*\)\s+String)\s*\)")
        if public_id.search(governed):
            fail(errors, f"{rel} exposes a public authority ID tuple field")
        bool_api = re.compile(r"pub\s+fn\s+\w+\s*\([^)]*\)\s*->\s*bool")
        if bool_api.search(governed):
            fail(errors, f"{rel} exposes a boolean-only public authority API")
        from_json = re.compile(r"pub\s+fn\s+from_json\s*\([^)]*\)\s*->\s*Result<")
        if not from_json.search(governed):
            fail(errors, f"{rel} must expose from_json returning Result<..., ...>")
        if "from_json" in governed and ".validate()?" not in governed:
            fail(errors, f"{rel} from_json must validate before returning Ok")
    evidence = root / "crates/horoji-core/src/evidence.rs"
    if evidence.is_file():
        text = evidence.read_text(encoding="utf-8")
        for variant in ["AdmissibilityDecision", "Admissible", "Rejected", "ReviewRequired"]:
            if variant not in text:
                fail(errors, f"crates/horoji-core/src/evidence.rs missing {variant}")
        for field in ["gate: EvidenceKind", "summary: String", "candidate_id", "evidence_id"]:
            if field not in text:
                fail(errors, f"crates/horoji-core/src/evidence.rs missing failure-evidence field: {field}")
    combined = "\n".join(
        (root / rel).read_text(encoding="utf-8")
        for rel in AUTHORITY_RUST_MODULES
        if (root / rel).is_file()
    )
    for test_name in REQUIRED_RUST_TEST_NAMES:
        if test_name not in combined:
            fail(errors, f"missing required Rust regression test: {test_name}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate acceptability engineering governance pack consistency.")
    parser.add_argument("--root", default=".", help="repository root; default: current directory")
    parser.add_argument("--no-generate", action="store_true", help="do not emit generated governance-pack manifest")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    errors: list[str] = []

    require_files(root, REQUIRED_DOCS, errors)
    require_files(root, REQUIRED_SCHEMAS, errors)
    require_files(root, REQUIRED_SCRIPTS, errors)
    validate_schema_files(root, errors)
    validate_markdown_headings(root, errors)
    validate_agents_navigation(root, errors)
    validate_governance_text(root, errors)
    validate_invariants(root, errors)
    validate_phasemap(root, errors)
    validate_forbidden_phrases(root, errors)
    validate_cross_terms(root, errors)
    validate_file_size_and_shape(root, errors)
    validate_rust_authority_surface(root, errors)
    if not args.no_generate:
        validate_pack_manifest(root, errors)

    if errors:
        print("Governance validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("Governance validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

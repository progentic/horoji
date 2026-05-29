<div align="center">

# Horoji

**A small governance harness for safer AI-assisted software work.**

[![Rust](https://img.shields.io/badge/Rust-ready-orange?logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-scripts-blue?logo=python)](https://www.python.org/)
[![Bash](https://img.shields.io/badge/Bash-checks-4EAA25?logo=gnubash&logoColor=white)](https://www.gnu.org/software/bash/)
[![JSON](https://img.shields.io/badge/JSON%20Schema-contracts-000000?logo=json)](https://json-schema.org/)

https://horoji.org

</div>

## What is Horoji?
Horoji is a production-ready acceptability engine that provides predictable guardrails for AI-driven software development. It enables engineering organizations to safely harness the velocity of AI agents and Large Language Models (LLMs) without exposing their core codebases to structural, architectural, or security risks.

The Golden Rule of AI Engineering: Generated code is untrusted by default. Horoji acts as an automated, zero-trust gatekeeper between your AI development tools and your production environment. The AI model may propose changes, but the Horoji harness autonomously evaluates, verifies, and certifies those proposals against your organization’s strict business rules before a single line of code reaches human review.

## The Strategic Value: Why It Matters

AI generates code at unprecedented speeds, but speed without guardrails introduces systemic risk. Left unchecked, autonomous models can inadvertently introduce hidden security vulnerabilities, ignore core system architecture, bypass testing standards, or alter out-of-scope files.

Horoji eliminates this anxiety by decoupling code generation from system authority. It answers the critical compliance questions automatically:

- **Intent Realization:** Is the AI actually doing what the user requested?

- **Blast Radius Control:** Is the AI restricted only to files it is explicitly permitted to modify?

- **Architectural Integrity:** Does the generated code adhere to your team’s strict structural, size, and complexity standards?

- **Compliance & Auditability:** Is there an unalterable, data-backed evidence trail showing exactly why this change was deemed safe?

## Operational Workflow

```text
                 Horoji acceptability workflow

 User intent
     |
     v
 Bounded change contract
     |
     v
 AI-generated candidate
     |
     v
 Harness checks
     |
     +--> rejected output --> rejection telemetry --> next attempt
     |
     v
 Admissible change
     |
     v
 Evidence bundle + human review + merge-ready artifact
```

## Repo Layout

```text
docs/
  AGENTS.md          Short guide for agents and contributors.
  ARCHITECTURE.md    How the workflow fits together.
  GOVERNANCE.md      Who or what has authority.
  INVARIANTS.md      Rules that must always remain true.
  CODING_STYLE.md    How code should stay reviewable.
  PHASEMAP.md        Small maturity roadmap from L0 to L5.

json/
  bounded-change-contract.schema.json
  evidence-record.schema.json
  rejection-telemetry.schema.json
  counterexample.schema.json
  governance-pack.schema.json
  governance-pack.generated.json

scripts/
  check.sh
  validate_governance.py
  validate_json_instance.py
```

## Example Uses

Use Horoji when you want AI-assisted development without trusting raw AI output.

Examples:

- A Rust service where AI can propose changes, but `cargo fmt`, `clippy`, `cargo test`, and dependency policy must pass first.
- A security tool where generated code must never add network access unless the change contract allows it.
- A regulated workflow where every accepted change needs an evidence record.
- A team using AI for refactors, but wanting hard limits on file size, function size, nesting, and schema depth.
- A project where rejected attempts should become counterexamples for future prompts and tests.

## Run the checks

From the repository root:

```bash
./scripts/check.sh .
```

The check script validates the governance pack and enforces mechanical consistency rules.

It fails when required files are missing, required invariants disappear, files become too large, Python functions/classes become too large, JSON becomes too deeply nested, Markdown headings become too deep, or source nesting becomes too complex.

## Enforced Size Limits

Horoji includes hard limits so the repository does not drift into large god files.

Current limits include:

```text
MAX_FILE_LOC = 1200
MAX_PY_FUNCTION_LOC = 180
MAX_PY_CLASS_LOC = 500
MAX_PY_BLOCK_NESTING = 5
MAX_JSON_DEPTH = 8
MAX_MARKDOWN_HEADING_DEPTH = 4
MAX_BRACE_NESTING = 6
```

These are not style preferences. They are enforced constraints.

## Rust Example Workflow

A common Rust workflow could look like this:

```text
1. Write a bounded change contract.
2. Let the model propose a Rust diff.
3. Run formatting and lint checks.
4. Run tests and property tests.
5. Check dependency policy.
6. Execute in a sandbox if needed.
7. Save evidence.
8. Send only admissible changes to human review.
```

Useful Rust tools include `serde`, `thiserror`, `cargo fmt`, `clippy`, `cargo test`, `proptest`, and `cargo-deny`.

## License

Horoji is released under the MIT License. See [LICENSE](LICENSE).

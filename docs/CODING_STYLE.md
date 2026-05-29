# CODING_STYLE

## Purpose

Coding style exists to support admissibility, auditability, and review.

Generated code must be easy to inspect, easy to test, and easy to reject when it violates the boundary.

## Rust style

Use simple Rust first.

Prefer:

- Explicit types at authority boundaries
- Small modules
- Narrow public APIs
- Exhaustive enums for state
- Typed errors with `thiserror`
- Serialization contracts with `serde`
- Deterministic ordering for emitted evidence
- Pure functions where practical
- Tests close to the behavior they defend

Avoid:

- Hidden global state
- Unbounded side effects
- Network access in tests unless explicitly allowed
- Implicit filesystem writes
- Panics in governed paths
- Untyped stringly state
- Broad catch-all errors
- Generated code that weakens review clarity

## Boundary code

Boundary code must make authority visible.

Use explicit names for:

- Candidate
- Admissible
- Rejected
- Evidence
- PolicyViolation
- ConstraintViolation
- ReviewRequired
- PromotionBlocked

## Tests

Tests are part of the acceptability surface.

Required test types should be selected by risk:

- Unit tests for local behavior
- Integration tests for contract behavior
- Property tests for invariant behavior
- Regression tests for counterexamples
- Mutation tests for test strength
- Sandbox tests for untrusted execution

## Comments and docs

Comments should explain why a boundary exists, not restate obvious syntax.

Document:

- Invariants
- Threat assumptions
- Rejection reasons
- Review requirements
- Rollback assumptions

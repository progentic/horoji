# INVARIANTS

These rules must remain true across the system.

## Core invariants

1. The LLM is non-authoritative.
2. The harness owns the authority boundary.
3. Generated code is untrusted by default.
4. Every admissible change carries evidence.
5. Human judgment revises the acceptability surface.
6. Rejected artifacts must not cross into trusted state.
7. State transitions must be explicit and auditable.
8. Evidence must be structured enough for replay.
9. Counterexamples must be preserved when they reveal a boundary gap.
10. Deployment approval must not be inferred from generation success.

## Acceptability surface

The acceptability surface is the executable definition of "good enough to advance."

It may include:

- Typed schemas
- Data invariants
- Pre-conditions and post-conditions
- Temporal properties
- Positive examples
- Counterexamples
- Acceptance tests
- Property-based tests
- Mutation testing
- Static analysis
- Dependency policy
- Sandbox policy
- Human review criteria
- Rollback requirements
- Telemetry requirements

## Non-admissible output

A candidate is non-admissible when it:

- Violates scope
- Touches forbidden surfaces
- Fails schema validation
- Fails tests
- Fails policy checks
- Lacks required evidence
- Weakens an invariant
- Introduces unreviewed authority
- Claims readiness without proof

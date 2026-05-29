# PHASEMAP

This file defines a small maturity roadmap for acceptability engineering.

## L0 - Ungoverned Generation

Direct trust in raw model output.

Risk is severe. The model may produce plausible artifacts without evidence.

Minimum exit condition:

- Generated output is no longer accepted without review.

## L1 - Syntax and Schema Enforcement

Basic structural validation exists.

Required capabilities:

- Formatting
- Type checking
- Schema validation
- Basic tests

Minimum exit condition:

- Candidate artifacts must compile and satisfy declared schemas.

## L2 - Harness Integration

Execution is sandboxed and observable.

Required capabilities:

- Controlled execution
- Dependency policy
- Import policy
- Sandbox logs
- Rejection telemetry

Minimum exit condition:

- Failed candidates produce structured rejection records.

## L3 - Semantic Loop

Telemetry and review findings revise the acceptability surface.

Required capabilities:

- Counterexample capture
- Incident feedback
- Review objection tracking
- Constraint revision

Minimum exit condition:

- Observed gaps become new tests, schemas, policies, or invariants.

## L4 - Actively Constrained

The harness tests the strength of the boundary.

Required capabilities:

- Mutation testing
- Property-based testing
- Structural drift detection
- Architecture policy checks

Minimum exit condition:

- Weak tests and architectural drift can trigger rejection.

## L5 - Mature Admissibility

Trust resides in the acceptability surface, evidence chain, and review process, not in any generator.

Required capabilities:

- Invariant-centric checks
- Zero-trust candidate execution
- Replayable evidence
- Strong provenance
- Human risk sign-off
- Boundary stress tests

Minimum exit condition:

- Every admissible change carries enough evidence to replay why it advanced.

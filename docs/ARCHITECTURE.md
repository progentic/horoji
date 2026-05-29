# ARCHITECTURE

## System model

This repository follows an acceptability engineering workflow.

Human intent is converted into a bounded change contract. The LLM proposes candidate Rust artifacts inside that constraint envelope. The Rust harness evaluates the candidate against the acceptability surface. Only admissible changes may advance.

## Workflow

```text
User Input
  -> Bounded Change Contract
  -> LLM Proposal Layer
  -> Rust Harness Orchestrator
  -> Admissible Rust Change
  -> Final Output
```

Rejected candidates return to the proposal layer with structured rejection telemetry.

Runtime evidence returns to the bounded change contract through the feedback loop.

## Bounded change contract

The bounded change contract is the primary artifact for a change.

It defines:

- Intent
- Scope
- Allowed surfaces
- Forbidden surfaces
- Rust crate or module boundaries
- Input and output schema
- Acceptance criteria
- Quality gates
- Review evidence
- Rollback expectations

## Rust harness orchestrator

The harness runs checks in a controlled order.

```text
1. Schema / contract validation
2. Static checks
3. Build and dependency policy
4. Test execution
5. Property testing
6. Mutation / adversarial checks
7. Sandboxed execution
8. Evidence capture
9. Human review gate
```

Example Rust tools:

- `serde` for typed serialization contracts
- JSON Schema for portable contract validation
- `cargo fmt` for formatting
- `clippy` for static checks
- `cargo build` for compilation
- `cargo deny` for dependency policy
- `cargo test` for tests
- `proptest` or `quickcheck` for property-based testing
- sandboxed execution for untrusted candidate runs

## Output classes

### Admissible Rust change

An admissible change:

- Compiles
- Passes required gates
- Carries evidence
- Has review approval
- Is safe to merge or promote under the current boundary

### Non-admissible output

A non-admissible output:

- Fails tests or policy
- Violates the boundary
- Lacks evidence
- Returns for correction with structured telemetry

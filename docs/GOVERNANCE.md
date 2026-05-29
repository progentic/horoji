# GOVERNANCE

## Purpose

This document defines the authority model for acceptability engineering.

Acceptability engineering is the discipline of defining, enforcing, and evolving the boundary between generated candidates and trusted system change.

The LLM searches the solution space. The harness rejects non-admissible artifacts. Human judgment decides whether the boundary itself remains correct.

## Authority model

The model is not authoritative.

The harness owns mechanical admissibility.

Humans own semantic judgment, risk acceptance, and boundary revision.

Generated artifacts are candidates only. A candidate becomes trusted system state only after it satisfies the current acceptability surface and passes review requirements.

## Boundary function

Let `S` be human intent, operational context, and risk constraints.

Let `G` be generation context, including repository state, instructions, examples, counterexamples, and prior feedback.

Let `C` be the space of possible executable artifacts.

Let `A` be the acceptability surface: schemas, invariants, tests, policy gates, review criteria, telemetry requirements, rollback rules, and architectural constraints.

The LLM proposes candidates:

```text
LLM: S x G -> C*
```

The harness evaluates candidates:

```text
B: C x A -> {0, 1}
```

```text
B(c, A) = 1 -> admissible
B(c, A) = 0 -> rejected + telemetry T(c)
```

## Governance rules

1. Treat all generated output as untrusted by default.
2. No candidate may advance without evidence.
3. No evidence may replace human judgment where meaning, risk, or strategic fit is unresolved.
4. Rejection telemetry must preserve the reason for rejection.
5. Counterexamples are governance assets.
6. The acceptability surface must evolve through review, incidents, telemetry, and escaped defects.
7. Claims about readiness, deployment, safety, or correctness require explicit evidence.

## Evidence chain

Every admissible artifact should carry evidence showing:

- What was generated
- Which constraints applied
- Which checks passed
- Which checks failed and were remediated
- Which reviewer accepted the residual risk
- Whether rollback is available
- Whether the decision can be replayed

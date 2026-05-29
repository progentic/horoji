# AGENTS

This repository treats generated output as untrusted until admitted by the harness.

Use this file only as a navigation map. Normative rules live in `/docs`.

## Read order

1. `/docs/GOVERNANCE.md`
2. `/docs/INVARIANTS.md`
3. `/docs/ARCHITECTURE.md`
4. `/docs/CODING_STYLE.md`
5. `/docs/PHASEMAP.md`

## Agent operating rule

An agent may propose changes, but it may not claim authority, readiness, deployment approval, or correctness unless the relevant governance file and harness evidence explicitly support that claim.

Every proposed change must identify:

- Intent
- Scope
- Allowed surfaces
- Forbidden surfaces
- Expected evidence
- Review boundary

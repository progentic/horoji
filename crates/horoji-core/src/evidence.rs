//! Evidence records decide whether a candidate can cross the authority boundary.
//!
//! Evidence is deterministic, ordered, and structured so rejection telemetry can
//! be replayed, audited, and safely returned to a proposal layer.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::contract::{BoundedChangeContract, CandidateId, QualityGate};

const MIN_ID_LEN: usize = 3;
const MAX_ID_LEN: usize = 96;

/// Identifies a deterministic evidence record for one candidate.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EvidenceId(pub(self) String);

/// Identifies a positive admissibility decision without exposing raw strings.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AcceptanceId(pub(self) String);

/// Names the kind of evidence attached to a gate decision.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    SchemaValidation,
    StaticAnalysis,
    Build,
    DependencyPolicy,
    UnitTest,
    PropertyTest,
    MutationTest,
    SandboxRun,
    HumanReview,
    RollbackCheck,
}

/// Provides an explicit gate state for deterministic evidence replay.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Passed,
    Failed,
    NotRun,
    Waived,
}

/// Separates human-review state from mechanical rejection.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    NotRequired,
    Required,
    Approved,
    Rejected,
}

/// Summarizes the whole evidence record without replacing detailed reasons.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverallStatus {
    Candidate,
    Admissible,
    Rejected,
    ReviewRequired,
}

/// One gate result with enough data to audit failed promotion attempts.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GateEvidence {
    pub gate: EvidenceKind,
    pub status: GateStatus,
    pub summary: String,
    pub candidate_id: CandidateId,
    pub evidence_id: EvidenceId,
}

/// Complete deterministic evidence for one candidate and contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceRecord {
    pub evidence_id: EvidenceId,
    pub acceptance_id: Option<AcceptanceId>,
    pub candidate_id: CandidateId,
    pub gate_results: BTreeSet<GateEvidence>,
    pub review_state: ReviewState,
    pub overall_status: OverallStatus,
    pub attestations: BTreeMap<String, String>,
}

/// Typed evidence violations used for rejection and review telemetry.
#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize)]
pub enum EvidenceViolation {
    #[error("invalid evidence id: {0}")]
    InvalidEvidenceId(String),
    #[error("invalid acceptance id: {0}")]
    InvalidAcceptanceId(String),
    #[error("evidence candidate does not match contract candidate")]
    CandidateMismatch,
    #[error("evidence must include at least one gate result")]
    EmptyGateResults,
    #[error("gate failed: {gate:?} for candidate {candidate_id:?} in evidence {evidence_id:?}: {summary}")]
    GateFailed {
        gate: EvidenceKind,
        summary: String,
        candidate_id: CandidateId,
        evidence_id: EvidenceId,
    },
    #[error("gate was not run: {0:?}")]
    GateNotRun(EvidenceKind),
    #[error("required contract gate missing from evidence: {0:?}")]
    MissingRequiredGate(QualityGate),
    #[error("human review is required")]
    ReviewRequired,
    #[error("human review rejected this candidate")]
    ReviewRejected,
    #[error("admissible status requires an acceptance id")]
    MissingAcceptanceId,
    #[error("evidence json is invalid: {0}")]
    InvalidJson(String),
}

/// Authority boundary result. This enum replaces boolean-only decisions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum AdmissibilityDecision {
    Admissible {
        acceptance_id: AcceptanceId,
        evidence_id: EvidenceId,
    },
    Rejected {
        violations: Vec<EvidenceViolation>,
    },
    ReviewRequired {
        reasons: Vec<EvidenceViolation>,
    },
}

impl EvidenceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AcceptanceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for EvidenceId {
    type Error = EvidenceViolation;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_id(value)
            .then(|| Self(value.to_owned()))
            .ok_or_else(|| EvidenceViolation::InvalidEvidenceId(value.to_owned()))
    }
}

impl TryFrom<&str> for AcceptanceId {
    type Error = EvidenceViolation;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_id(value)
            .then(|| Self(value.to_owned()))
            .ok_or_else(|| EvidenceViolation::InvalidAcceptanceId(value.to_owned()))
    }
}

impl FromStr for EvidenceId {
    type Err = EvidenceViolation;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl FromStr for AcceptanceId {
    type Err = EvidenceViolation;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl Serialize for EvidenceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl Serialize for AcceptanceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EvidenceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(EvidenceIdVisitor)
    }
}

impl<'de> Deserialize<'de> for AcceptanceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AcceptanceIdVisitor)
    }
}

struct EvidenceIdVisitor;

impl Visitor<'_> for EvidenceIdVisitor {
    type Value = EvidenceId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a validated evidence id")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        EvidenceId::try_from(value).map_err(E::custom)
    }
}

struct AcceptanceIdVisitor;

impl Visitor<'_> for AcceptanceIdVisitor {
    type Value = AcceptanceId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a validated acceptance id")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        AcceptanceId::try_from(value).map_err(E::custom)
    }
}

impl EvidenceRecord {
    pub fn builder(evidence_id: EvidenceId, candidate_id: CandidateId) -> EvidenceRecordBuilder {
        EvidenceRecordBuilder {
            evidence_id,
            acceptance_id: None,
            candidate_id,
            gate_results: BTreeSet::new(),
            review_state: ReviewState::Required,
            overall_status: OverallStatus::Candidate,
            attestations: BTreeMap::new(),
        }
    }

    pub fn from_json(input: &str) -> Result<Self, EvidenceViolation> {
        let evidence: Self = serde_json::from_str(input)
            .map_err(|error| EvidenceViolation::InvalidJson(error.to_string()))?;
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn validate(&self) -> Result<(), EvidenceViolation> {
        if self.gate_results.is_empty() {
            return Err(EvidenceViolation::EmptyGateResults);
        }
        if self.overall_status == OverallStatus::Admissible && self.acceptance_id.is_none() {
            return Err(EvidenceViolation::MissingAcceptanceId);
        }
        if self.review_state == ReviewState::Rejected {
            return Err(EvidenceViolation::ReviewRejected);
        }
        if self.review_state == ReviewState::Required {
            return Err(EvidenceViolation::ReviewRequired);
        }
        self.failed_gates().into_iter().next().map_or(Ok(()), Err)
    }

    pub fn failed_gates(&self) -> Vec<EvidenceViolation> {
        self.gate_results
            .iter()
            .filter_map(|gate| match gate.status {
                GateStatus::Passed | GateStatus::Waived => None,
                GateStatus::Failed => Some(EvidenceViolation::GateFailed {
                    gate: gate.gate.clone(),
                    summary: gate.summary.clone(),
                    candidate_id: gate.candidate_id.clone(),
                    evidence_id: gate.evidence_id.clone(),
                }),
                GateStatus::NotRun => Some(EvidenceViolation::GateNotRun(gate.gate.clone())),
            })
            .collect()
    }

    pub fn admissibility(&self, contract: &BoundedChangeContract) -> AdmissibilityDecision {
        let violations = self.admissibility_violations(contract);
        if !violations.is_empty() {
            return AdmissibilityDecision::Rejected { violations };
        }
        match (&self.acceptance_id, &self.review_state) {
            (_, ReviewState::Required) => AdmissibilityDecision::ReviewRequired {
                reasons: vec![EvidenceViolation::ReviewRequired],
            },
            (Some(acceptance_id), _) => AdmissibilityDecision::Admissible {
                acceptance_id: acceptance_id.clone(),
                evidence_id: self.evidence_id.clone(),
            },
            (None, _) => AdmissibilityDecision::Rejected {
                violations: vec![EvidenceViolation::MissingAcceptanceId],
            },
        }
    }

    fn admissibility_violations(&self, contract: &BoundedChangeContract) -> Vec<EvidenceViolation> {
        let mut violations = Vec::new();
        if self.candidate_id != contract.candidate_id {
            violations.push(EvidenceViolation::CandidateMismatch);
        }
        violations.extend(self.missing_required_gates(contract));
        violations.extend(self.failed_gates());
        if self.review_state == ReviewState::Rejected {
            violations.push(EvidenceViolation::ReviewRejected);
        }
        violations
    }

    fn missing_required_gates(&self, contract: &BoundedChangeContract) -> Vec<EvidenceViolation> {
        let present = self.present_quality_gates();
        contract
            .required_quality_gates
            .iter()
            .filter(|gate| !present.contains(*gate))
            .cloned()
            .map(EvidenceViolation::MissingRequiredGate)
            .collect()
    }

    fn present_quality_gates(&self) -> BTreeSet<QualityGate> {
        self.gate_results
            .iter()
            .filter_map(|gate| gate.gate.as_quality_gate())
            .collect()
    }
}

impl EvidenceKind {
    fn as_quality_gate(&self) -> Option<QualityGate> {
        match self {
            Self::SchemaValidation => Some(QualityGate::SchemaValidation),
            Self::StaticAnalysis => Some(QualityGate::CargoClippy),
            Self::Build => Some(QualityGate::CargoBuild),
            Self::DependencyPolicy => Some(QualityGate::CargoDeny),
            Self::UnitTest => Some(QualityGate::CargoTest),
            Self::PropertyTest => Some(QualityGate::PropertyTest),
            Self::MutationTest => Some(QualityGate::MutationTest),
            Self::SandboxRun => Some(QualityGate::SandboxedExecution),
            Self::HumanReview => Some(QualityGate::HumanReview),
            Self::RollbackCheck => None,
        }
    }
}

/// Builder keeps evidence assembly explicit and ordered.
pub struct EvidenceRecordBuilder {
    evidence_id: EvidenceId,
    acceptance_id: Option<AcceptanceId>,
    candidate_id: CandidateId,
    gate_results: BTreeSet<GateEvidence>,
    review_state: ReviewState,
    overall_status: OverallStatus,
    attestations: BTreeMap<String, String>,
}

impl EvidenceRecordBuilder {
    pub fn acceptance_id(mut self, acceptance_id: AcceptanceId) -> Self {
        self.acceptance_id = Some(acceptance_id);
        self
    }

    pub fn gate_result(mut self, gate: GateEvidence) -> Self {
        self.gate_results.insert(gate);
        self
    }

    pub fn review_state(mut self, review_state: ReviewState) -> Self {
        self.review_state = review_state;
        self
    }

    pub fn overall_status(mut self, overall_status: OverallStatus) -> Self {
        self.overall_status = overall_status;
        self
    }

    pub fn attestation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attestations.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<EvidenceRecord, EvidenceViolation> {
        let evidence = EvidenceRecord {
            evidence_id: self.evidence_id,
            acceptance_id: self.acceptance_id,
            candidate_id: self.candidate_id,
            gate_results: self.gate_results,
            review_state: self.review_state,
            overall_status: self.overall_status,
            attestations: self.attestations,
        };
        evidence.validate()?;
        Ok(evidence)
    }
}

fn validate_id(value: &str) -> bool {
    let valid_len = (MIN_ID_LEN..=MAX_ID_LEN).contains(&value.len());
    valid_len && value.chars().all(valid_id_char)
}

fn valid_id_char(value: char) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || matches!(value, '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{BoundedChangeContract, ChangeType, ContractId};

    fn candidate_id() -> CandidateId {
        CandidateId::try_from("candidate-001").expect("valid candidate id")
    }

    fn contract() -> BoundedChangeContract {
        BoundedChangeContract::builder(
            ContractId::try_from("contract-001").expect("valid contract id"),
            candidate_id(),
            ChangeType::Feature,
            "add evidence boundary",
        )
        .allowed_surface("crates/horoji-core/src/evidence.rs")
        .schema_ref("json/evidence-record.schema.json")
        .quality_gate(QualityGate::CargoTest)
        .build()
        .expect("valid contract")
    }

    fn passed_gate(kind: EvidenceKind) -> GateEvidence {
        GateEvidence {
            gate: kind,
            status: GateStatus::Passed,
            summary: "passed".to_owned(),
            candidate_id: candidate_id(),
            evidence_id: EvidenceId::try_from("evidence-001").expect("valid evidence id"),
        }
    }

    fn admissible_evidence() -> EvidenceRecord {
        EvidenceRecord::builder(
            EvidenceId::try_from("evidence-001").expect("valid evidence id"),
            candidate_id(),
        )
        .acceptance_id(AcceptanceId::try_from("acceptance-001").expect("valid acceptance id"))
        .gate_result(passed_gate(EvidenceKind::UnitTest))
        .review_state(ReviewState::Approved)
        .overall_status(OverallStatus::Admissible)
        .attestation("cargo_test", "passed")
        .build()
        .expect("valid evidence")
    }

    #[test]
    fn evidence_id_rejects_invalid() {
        assert!(EvidenceId::try_from("Bad ID").is_err());
    }

    #[test]
    fn admissible_evidence_passes_check() {
        let decision = admissible_evidence().admissibility(&contract());
        assert!(matches!(decision, AdmissibilityDecision::Admissible { .. }));
    }

    #[test]
    fn failed_gate_blocks_promotion() {
        let failed = GateEvidence {
            gate: EvidenceKind::UnitTest,
            status: GateStatus::Failed,
            summary: "unit tests failed".to_owned(),
            candidate_id: candidate_id(),
            evidence_id: EvidenceId::try_from("evidence-002").expect("valid evidence id"),
        };
        let evidence = EvidenceRecord::builder(
            EvidenceId::try_from("evidence-002").expect("valid evidence id"),
            candidate_id(),
        )
        .acceptance_id(AcceptanceId::try_from("acceptance-002").expect("valid acceptance id"))
        .gate_result(failed)
        .review_state(ReviewState::Approved)
        .overall_status(OverallStatus::Rejected)
        .build();
        assert!(matches!(evidence, Err(EvidenceViolation::GateFailed { .. })));
    }

    #[test]
    fn review_required_is_distinct_from_rejected() {
        let evidence = EvidenceRecord {
            evidence_id: EvidenceId::try_from("evidence-003").expect("valid evidence id"),
            acceptance_id: Some(AcceptanceId::try_from("acceptance-003").expect("valid acceptance id")),
            candidate_id: candidate_id(),
            gate_results: BTreeSet::from([passed_gate(EvidenceKind::UnitTest)]),
            review_state: ReviewState::Required,
            overall_status: OverallStatus::ReviewRequired,
            attestations: BTreeMap::new(),
        };
        let decision = evidence.admissibility(&contract());
        assert!(matches!(decision, AdmissibilityDecision::ReviewRequired { .. }));
    }

    #[test]
    fn evidence_json_is_stable_when_gate_insertion_order_changes() {
        let first = admissible_evidence();
        let second = EvidenceRecord::builder(
            EvidenceId::try_from("evidence-001").expect("valid evidence id"),
            candidate_id(),
        )
        .gate_result(passed_gate(EvidenceKind::UnitTest))
        .attestation("cargo_test", "passed")
        .acceptance_id(AcceptanceId::try_from("acceptance-001").expect("valid acceptance id"))
        .review_state(ReviewState::Approved)
        .overall_status(OverallStatus::Admissible)
        .build()
        .expect("valid evidence");
        let first_json = serde_json::to_string_pretty(&first).expect("serialize first");
        let second_json = serde_json::to_string_pretty(&second).expect("serialize second");
        assert_eq!(first_json, second_json);
    }
}

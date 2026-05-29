//! Bounded change contracts define the acceptability surface.
//!
//! A contract is the explicit boundary a generated candidate must satisfy
//! before evidence can argue for promotion. IDs are private newtypes so raw
//! strings cannot bypass validation at authority boundaries.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

const MIN_ID_LEN: usize = 3;
const MAX_ID_LEN: usize = 96;

/// Identifies a bounded change contract and prevents arbitrary string IDs.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContractId(pub(self) String);

/// Identifies a generated candidate and ties evidence back to one proposal.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CandidateId(pub(self) String);

/// Classifies the kind of system change being proposed.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Feature,
    Refactor,
    BugFix,
    SecurityFix,
    Documentation,
    Governance,
}

/// Names a required quality gate without using stringly typed gate status.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityGate {
    SchemaValidation,
    CargoFmt,
    CargoClippy,
    CargoBuild,
    CargoDeny,
    CargoTest,
    PropertyTest,
    MutationTest,
    SandboxedExecution,
    HumanReview,
}

/// Contract a candidate must satisfy before evidence can support promotion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BoundedChangeContract {
    pub contract_id: ContractId,
    pub candidate_id: CandidateId,
    pub change_type: ChangeType,
    pub intent: String,
    pub allowed_surfaces: BTreeSet<String>,
    pub forbidden_patterns: BTreeSet<String>,
    pub input_output_schema_refs: BTreeSet<String>,
    pub required_quality_gates: BTreeSet<QualityGate>,
    pub risk_notes: BTreeSet<String>,
}

/// Typed contract violations used before a contract becomes trusted state.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ContractViolation {
    #[error("invalid contract id: {0}")]
    InvalidContractId(String),
    #[error("invalid candidate id: {0}")]
    InvalidCandidateId(String),
    #[error("intent must not be empty")]
    EmptyIntent,
    #[error("allowed surfaces must not be empty")]
    EmptyAllowedSurfaces,
    #[error("required quality gates must not be empty")]
    EmptyQualityGates,
    #[error("schema references must not be empty")]
    EmptySchemaReferences,
    #[error("contract json is invalid: {0}")]
    InvalidJson(String),
}

impl ContractId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CandidateId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ContractId {
    type Error = ContractViolation;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_id(value)
            .then(|| Self(value.to_owned()))
            .ok_or_else(|| ContractViolation::InvalidContractId(value.to_owned()))
    }
}

impl TryFrom<&str> for CandidateId {
    type Error = ContractViolation;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_id(value)
            .then(|| Self(value.to_owned()))
            .ok_or_else(|| ContractViolation::InvalidCandidateId(value.to_owned()))
    }
}

impl FromStr for ContractId {
    type Err = ContractViolation;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl FromStr for CandidateId {
    type Err = ContractViolation;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl Serialize for ContractId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl Serialize for CandidateId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ContractId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ContractIdVisitor)
    }
}

impl<'de> Deserialize<'de> for CandidateId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CandidateIdVisitor)
    }
}

struct ContractIdVisitor;

impl Visitor<'_> for ContractIdVisitor {
    type Value = ContractId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a validated contract id")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ContractId::try_from(value).map_err(E::custom)
    }
}

struct CandidateIdVisitor;

impl Visitor<'_> for CandidateIdVisitor {
    type Value = CandidateId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a validated candidate id")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        CandidateId::try_from(value).map_err(E::custom)
    }
}

impl BoundedChangeContract {
    pub fn builder(
        contract_id: ContractId,
        candidate_id: CandidateId,
        change_type: ChangeType,
        intent: impl Into<String>,
    ) -> BoundedChangeContractBuilder {
        BoundedChangeContractBuilder {
            contract_id,
            candidate_id,
            change_type,
            intent: intent.into(),
            allowed_surfaces: BTreeSet::new(),
            forbidden_patterns: BTreeSet::new(),
            input_output_schema_refs: BTreeSet::new(),
            required_quality_gates: BTreeSet::new(),
            risk_notes: BTreeSet::new(),
        }
    }

    pub fn from_json(input: &str) -> Result<Self, ContractViolation> {
        let contract: Self = serde_json::from_str(input)
            .map_err(|error| ContractViolation::InvalidJson(error.to_string()))?;
        contract.validate()?;
        Ok(contract)
    }

    pub fn validate(&self) -> Result<(), ContractViolation> {
        if self.intent.trim().is_empty() {
            return Err(ContractViolation::EmptyIntent);
        }
        if self.allowed_surfaces.is_empty() {
            return Err(ContractViolation::EmptyAllowedSurfaces);
        }
        if self.required_quality_gates.is_empty() {
            return Err(ContractViolation::EmptyQualityGates);
        }
        if self.input_output_schema_refs.is_empty() {
            return Err(ContractViolation::EmptySchemaReferences);
        }
        Ok(())
    }
}

/// Builder keeps construction explicit while validation remains the boundary.
pub struct BoundedChangeContractBuilder {
    contract_id: ContractId,
    candidate_id: CandidateId,
    change_type: ChangeType,
    intent: String,
    allowed_surfaces: BTreeSet<String>,
    forbidden_patterns: BTreeSet<String>,
    input_output_schema_refs: BTreeSet<String>,
    required_quality_gates: BTreeSet<QualityGate>,
    risk_notes: BTreeSet<String>,
}

impl BoundedChangeContractBuilder {
    pub fn allowed_surface(mut self, surface: impl Into<String>) -> Self {
        self.allowed_surfaces.insert(surface.into());
        self
    }

    pub fn forbidden_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.forbidden_patterns.insert(pattern.into());
        self
    }

    pub fn schema_ref(mut self, schema_ref: impl Into<String>) -> Self {
        self.input_output_schema_refs.insert(schema_ref.into());
        self
    }

    pub fn quality_gate(mut self, gate: QualityGate) -> Self {
        self.required_quality_gates.insert(gate);
        self
    }

    pub fn risk_note(mut self, note: impl Into<String>) -> Self {
        self.risk_notes.insert(note.into());
        self
    }

    pub fn build(self) -> Result<BoundedChangeContract, ContractViolation> {
        let contract = BoundedChangeContract {
            contract_id: self.contract_id,
            candidate_id: self.candidate_id,
            change_type: self.change_type,
            intent: self.intent,
            allowed_surfaces: self.allowed_surfaces,
            forbidden_patterns: self.forbidden_patterns,
            input_output_schema_refs: self.input_output_schema_refs,
            required_quality_gates: self.required_quality_gates,
            risk_notes: self.risk_notes,
        };
        contract.validate()?;
        Ok(contract)
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

    fn contract() -> BoundedChangeContract {
        BoundedChangeContract::builder(
            ContractId::try_from("contract-001").expect("valid contract id"),
            CandidateId::try_from("candidate-001").expect("valid candidate id"),
            ChangeType::Feature,
            "add governed rust validation",
        )
        .allowed_surface("crates/horoji-core/src/contract.rs")
        .schema_ref("json/bounded-change-contract.schema.json")
        .quality_gate(QualityGate::CargoTest)
        .build()
        .expect("valid contract")
    }

    #[test]
    fn contract_id_rejects_invalid() {
        assert!(ContractId::try_from("Bad ID").is_err());
    }

    #[test]
    fn empty_allowed_surfaces_rejected() {
        let result = BoundedChangeContract::builder(
            ContractId::try_from("contract-002").expect("valid contract id"),
            CandidateId::try_from("candidate-002").expect("valid candidate id"),
            ChangeType::Feature,
            "missing allowed surfaces",
        )
        .schema_ref("json/bounded-change-contract.schema.json")
        .quality_gate(QualityGate::CargoTest)
        .build();
        assert_eq!(result, Err(ContractViolation::EmptyAllowedSurfaces));
    }

    #[test]
    fn from_json_rejects_structurally_valid_but_policy_invalid_contract() {
        let input = r#"{
            "contract_id":"contract-003",
            "candidate_id":"candidate-003",
            "change_type":"feature",
            "intent":"missing quality gates",
            "allowed_surfaces":["crates/horoji-core/src/contract.rs"],
            "forbidden_patterns":[],
            "input_output_schema_refs":["json/bounded-change-contract.schema.json"],
            "required_quality_gates":[],
            "risk_notes":[]
        }"#;
        let result = BoundedChangeContract::from_json(input);
        assert_eq!(result, Err(ContractViolation::EmptyQualityGates));
    }

    #[test]
    fn contract_json_is_stable_across_repeated_serialization() {
        let value = contract();
        let first = serde_json::to_string_pretty(&value).expect("serialize contract");
        let second = serde_json::to_string_pretty(&value).expect("serialize contract again");
        assert_eq!(first, second);
    }
}

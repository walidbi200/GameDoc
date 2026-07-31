//! Append-only observations and the immutable fact store.

use std::cmp::Ordering;

use serde::Serialize;
use serde_json::Value;

use crate::ids::{CollectorId, FactId};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    System,
    Cpu,
    Distro,
    Gpu,
    Steam,
    MangoHud,
    GameMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactConfidence {
    Detected,
    NotDetected,
    Unknown,
    Error,
}

impl std::fmt::Display for FactConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Detected => "detected",
            Self::NotDetected => "not_detected",
            Self::Unknown => "unknown",
            Self::Error => "error",
        };
        f.write_str(label)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FactSource {
    pub collector: CollectorId,
    pub method: ObservationMethod,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "target", rename_all = "snake_case")]
pub enum ObservationMethod {
    Command(String),
    File(String),
    Environment(String),
    Path(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Fact {
    pub id: FactId,
    pub category: Category,
    pub value: Value,
    pub confidence: FactConfidence,
    pub source: FactSource,
}

impl Fact {
    #[must_use]
    pub fn observed(
        id: FactId,
        category: Category,
        value: impl Into<Value>,
        source: FactSource,
    ) -> Self {
        Self {
            id,
            category,
            value: value.into(),
            confidence: FactConfidence::Detected,
            source,
        }
    }
}

#[derive(Debug, Default)]
pub struct FactBuilder {
    facts: Vec<Fact>,
}

impl FactBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, facts: impl IntoIterator<Item = Fact>) {
        self.facts.extend(facts);
    }

    #[must_use]
    pub fn build(mut self) -> Facts {
        self.facts.sort_by(compare_facts);
        Facts { facts: self.facts }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Facts {
    facts: Vec<Fact>,
}

impl Facts {
    #[must_use]
    pub fn all(&self, id: FactId) -> Vec<&Fact> {
        self.facts.iter().filter(|fact| fact.id == id).collect()
    }

    #[must_use]
    pub fn first(&self, id: FactId) -> Option<&Fact> {
        self.facts.iter().find(|fact| fact.id == id)
    }

    #[must_use]
    pub fn count(&self, id: FactId) -> usize {
        self.facts.iter().filter(|fact| fact.id == id).count()
    }

    #[must_use]
    pub fn exists(&self, id: FactId) -> bool {
        self.first(id).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Fact> {
        self.facts.iter()
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<Fact> {
        self.facts
    }
}

fn compare_facts(left: &Fact, right: &Fact) -> Ordering {
    left.category
        .cmp(&right.category)
        .then_with(|| left.id.cmp(&right.id))
        .then_with(|| canonical_value(&left.value).cmp(&canonical_value(&right.value)))
}

fn canonical_value(value: &Value) -> String {
    serde_json::to_string(value).expect("JSON values are always serializable")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Category, Fact, FactBuilder, FactSource, ObservationMethod};
    use crate::ids::{collector, fact};

    fn gpu(value: &str) -> Fact {
        Fact::observed(
            fact::GPU_DEVICE,
            Category::Gpu,
            value,
            FactSource {
                collector: collector::GRAPHICS,
                method: ObservationMethod::Command("lspci".to_owned()),
            },
        )
    }

    #[test]
    fn repeated_facts_are_retained_and_sorted() {
        let mut builder = FactBuilder::new();
        builder.append([gpu("Intel UHD"), gpu("AMD RX 6800")]);
        let facts = builder.build();

        assert_eq!(facts.count(fact::GPU_DEVICE), 2);
        assert_eq!(facts.all(fact::GPU_DEVICE)[0].value, json!("AMD RX 6800"));
    }

    #[test]
    fn built_facts_support_read_only_lookups() {
        let mut builder = FactBuilder::new();
        builder.append([gpu("AMD RX 6800")]);
        let facts = builder.build();

        assert!(facts.exists(fact::GPU_DEVICE));
        assert_eq!(
            facts.first(fact::GPU_DEVICE).unwrap().value,
            json!("AMD RX 6800")
        );
    }
}

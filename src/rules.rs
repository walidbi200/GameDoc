//! Pure rule evaluation over immutable facts.

use crate::facts::{Category, Facts};
use crate::ids::{fact, problem, rule, FactId, RuleId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleStatus {
    Passed,
    Problem,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleOutcome {
    pub rule_id: RuleId,
    pub status: RuleStatus,
    pub explanation: String,
    pub candidates: Vec<ProblemCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProblemCandidate {
    pub id: crate::ids::ProblemId,
    pub category: Category,
    pub evidence: Vec<String>,
}

pub trait Rule: Send + Sync {
    fn id(&self) -> RuleId;
    fn requires(&self) -> &'static [FactId];
    fn optional(&self) -> &'static [FactId];
    fn evaluate(&self, facts: &Facts) -> RuleOutcome;
}

#[must_use]
pub fn evaluate_all(facts: &Facts) -> Vec<RuleOutcome> {
    let mut rules: Vec<&dyn Rule> = vec![
        &VulkanAvailableRule,
        &SteamProtonRule,
        &SteamLibraryReadableRule,
        &GameModeDaemonRule,
        &CpuGovernorRule,
    ];
    rules.sort_by_key(|item| item.id());
    rules
        .into_iter()
        .map(|item| evaluate(item, facts))
        .collect()
}

fn evaluate(rule: &dyn Rule, facts: &Facts) -> RuleOutcome {
    let missing: Vec<&str> = rule
        .requires()
        .iter()
        .filter(|id| !facts.exists(**id))
        .map(|id| id.as_str())
        .collect();

    if missing.is_empty() {
        rule.evaluate(facts)
    } else {
        RuleOutcome {
            rule_id: rule.id(),
            status: RuleStatus::Skipped,
            explanation: format!("missing required facts: {}", missing.join(", ")),
            candidates: Vec::new(),
        }
    }
}

struct VulkanAvailableRule;

impl Rule for VulkanAvailableRule {
    fn id(&self) -> RuleId {
        rule::VULKAN_AVAILABLE
    }

    fn requires(&self) -> &'static [FactId] {
        &[]
    }

    fn optional(&self) -> &'static [FactId] {
        &[fact::GPU_DEVICE]
    }

    fn evaluate(&self, facts: &Facts) -> RuleOutcome {
        let item = facts.first(fact::VULKAN_AVAILABLE);

        match item {
            Some(fact) => match fact.value.as_bool() {
                Some(true) => RuleOutcome {
                    rule_id: self.id(),
                    status: RuleStatus::Passed,
                    explanation: "vulkaninfo completed successfully".to_owned(),
                    candidates: Vec::new(),
                },
                Some(false) => RuleOutcome {
                    rule_id: self.id(),
                    status: RuleStatus::Problem,
                    explanation: "vulkaninfo did not complete successfully".to_owned(),
                    candidates: vec![ProblemCandidate {
                        id: problem::VULKAN_UNAVAILABLE,
                        category: Category::Gpu,
                        evidence: vec!["The local Vulkan probe exited unsuccessfully.".to_owned()],
                    }],
                },
                None => RuleOutcome {
                    rule_id: self.id(),
                    status: RuleStatus::Skipped,
                    explanation: "gpu.vulkan_available was not a boolean".to_owned(),
                    candidates: Vec::new(),
                },
            },
            None => RuleOutcome {
                rule_id: self.id(),
                status: RuleStatus::Problem,
                explanation: "vulkaninfo was missing".to_owned(),
                candidates: vec![ProblemCandidate {
                    id: problem::VULKAN_UNAVAILABLE,
                    category: Category::Gpu,
                    evidence: vec!["The local Vulkan probe was missing.".to_owned()],
                }],
            },
        }
    }
}

struct SteamProtonRule;

impl Rule for SteamProtonRule {
    fn id(&self) -> RuleId {
        rule::STEAM_PROTON
    }

    fn requires(&self) -> &'static [FactId] {
        &[fact::STEAM_INSTALLATION]
    }

    fn optional(&self) -> &'static [FactId] {
        &[
            fact::STEAM_PROTON_VERSION,
            fact::STEAM_COMPATIBILITY_TOOL,
            fact::STEAM_PROTON_SCAN_COMPLETE,
        ]
    }

    fn evaluate(&self, facts: &Facts) -> RuleOutcome {
        if !facts
            .all(fact::STEAM_DATA_READABLE)
            .into_iter()
            .any(|item| item.value["readable"].as_bool() == Some(true))
        {
            return RuleOutcome {
                rule_id: self.id(),
                status: RuleStatus::Skipped,
                explanation: "Steam data was not readable enough to evaluate Proton".to_owned(),
                candidates: Vec::new(),
            };
        }
        if facts
            .all(fact::STEAM_PROTON_SCAN_COMPLETE)
            .into_iter()
            .any(|item| item.value["complete"].as_bool() != Some(true))
            || !facts.exists(fact::STEAM_PROTON_SCAN_COMPLETE)
        {
            return RuleOutcome {
                rule_id: self.id(),
                status: RuleStatus::Skipped,
                explanation: "Proton directories were not fully readable".to_owned(),
                candidates: Vec::new(),
            };
        }
        if facts.exists(fact::STEAM_PROTON_VERSION) || facts.exists(fact::STEAM_COMPATIBILITY_TOOL)
        {
            passed(
                self.id(),
                "At least one Proton runtime or compatibility tool was observed",
            )
        } else {
            problem(
                self.id(),
                "Steam was observed without a Proton runtime or compatibility tool",
                ProblemCandidate {
                    id: problem::STEAM_PROTON_MISSING,
                    category: Category::Steam,
                    evidence: vec![
                        "Steam is installed, but no Proton runtime was observed.".to_owned()
                    ],
                },
            )
        }
    }
}

struct SteamLibraryReadableRule;

impl Rule for SteamLibraryReadableRule {
    fn id(&self) -> RuleId {
        rule::STEAM_LIBRARY_READABLE
    }

    fn requires(&self) -> &'static [FactId] {
        &[fact::STEAM_INSTALLATION, fact::STEAM_LIBRARY_READABLE]
    }

    fn optional(&self) -> &'static [FactId] {
        &[]
    }

    fn evaluate(&self, facts: &Facts) -> RuleOutcome {
        let unreadable = facts
            .all(fact::STEAM_LIBRARY_READABLE)
            .into_iter()
            .filter(|item| item.value["readable"].as_bool() == Some(false))
            .count();
        if unreadable == 0 {
            passed(self.id(), "All observed Steam libraries were readable")
        } else {
            problem(
                self.id(),
                "At least one Steam library was unreadable",
                ProblemCandidate {
                    id: problem::STEAM_LIBRARY_UNREADABLE,
                    category: Category::Steam,
                    evidence: vec![format!(
                        "{unreadable} Steam library location(s) could not be read."
                    )],
                },
            )
        }
    }
}

struct GameModeDaemonRule;

impl Rule for GameModeDaemonRule {
    fn id(&self) -> RuleId {
        rule::GAMEMODE_DAEMON
    }

    fn requires(&self) -> &'static [FactId] {
        &[fact::GAMEMODE_EXECUTABLE, fact::GAMEMODE_DAEMON_AVAILABLE]
    }

    fn optional(&self) -> &'static [FactId] {
        &[fact::GAMEMODE_LIBRARY]
    }

    fn evaluate(&self, facts: &Facts) -> RuleOutcome {
        if facts
            .first(fact::GAMEMODE_DAEMON_AVAILABLE)
            .and_then(|item| item.value.as_bool())
            == Some(true)
        {
            passed(self.id(), "GameMode daemon status probe succeeded")
        } else {
            problem(
                self.id(),
                "GameMode was observed but its daemon status probe failed",
                ProblemCandidate {
                    id: problem::GAMEMODE_DAEMON_UNAVAILABLE,
                    category: Category::GameMode,
                    evidence: vec![
                        "GameMode is installed, but its daemon did not respond successfully."
                            .to_owned(),
                    ],
                },
            )
        }
    }
}

fn passed(rule_id: RuleId, explanation: &str) -> RuleOutcome {
    RuleOutcome {
        rule_id,
        status: RuleStatus::Passed,
        explanation: explanation.to_owned(),
        candidates: Vec::new(),
    }
}

fn problem(rule_id: RuleId, explanation: &str, candidate: ProblemCandidate) -> RuleOutcome {
    RuleOutcome {
        rule_id,
        status: RuleStatus::Problem,
        explanation: explanation.to_owned(),
        candidates: vec![candidate],
    }
}

/// Detects a `powersave` CPU frequency governor, which significantly reduces
/// gaming performance by keeping the CPU at minimum clock speed even under load.
///
/// Governors like `schedutil` and `performance` respond to load and are safe
/// for gaming. `powersave` is the default on many laptop distributions.
struct CpuGovernorRule;

impl Rule for CpuGovernorRule {
    fn id(&self) -> RuleId {
        rule::CPU_GOVERNOR
    }

    fn requires(&self) -> &'static [FactId] {
        &[fact::CPU_GOVERNOR]
    }

    fn optional(&self) -> &'static [FactId] {
        &[]
    }

    fn evaluate(&self, facts: &Facts) -> RuleOutcome {
        let governor = facts
            .first(fact::CPU_GOVERNOR)
            .and_then(|f| f.value.as_str().map(str::to_owned));

        match governor.as_deref() {
            Some("powersave") => problem(
                self.id(),
                "CPU governor is powersave — CPU stays at minimum frequency",
                ProblemCandidate {
                    id: problem::CPU_GOVERNOR_POWERSAVE,
                    category: Category::System,
                    evidence: vec![
                        "The CPU frequency governor is set to 'powersave'.".to_owned(),
                        "This keeps the CPU at its minimum clock frequency even under full \
                         load, which can reduce gaming performance by 20–50%."
                            .to_owned(),
                    ],
                },
            ),
            Some(_) => passed(self.id(), "CPU governor allows frequency scaling"),
            None => RuleOutcome {
                rule_id: self.id(),
                status: RuleStatus::Skipped,
                explanation: "CPU governor fact had an unexpected value type".to_owned(),
                candidates: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::facts::{Category, Fact, FactBuilder, FactSource, ObservationMethod};
    use crate::ids::{collector, fact, rule};

    use super::{evaluate_all, RuleStatus};

    #[test]
    fn missing_vulkan_fact_emits_problem() {
        let outcomes = evaluate_all(&FactBuilder::new().build());
        let outcome = outcomes
            .iter()
            .find(|item| item.rule_id == rule::VULKAN_AVAILABLE)
            .unwrap();

        assert_eq!(outcome.status, RuleStatus::Problem);
        assert_eq!(outcome.candidates.len(), 1);
    }

    #[test]
    fn explicit_failed_probe_produces_problem_outcome() {
        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::VULKAN_AVAILABLE,
            Category::Gpu,
            false,
            FactSource {
                collector: collector::GRAPHICS,
                method: ObservationMethod::Command("vulkaninfo --summary".to_owned()),
            },
        )]);

        let outcomes = evaluate_all(&builder.build());
        let outcome = outcomes
            .iter()
            .find(|item| item.rule_id == rule::VULKAN_AVAILABLE)
            .unwrap();

        assert_eq!(outcome.status, RuleStatus::Problem);
        assert_eq!(outcome.candidates.len(), 1);
    }

    #[test]
    fn malformed_required_fact_skips_rule() {
        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::VULKAN_AVAILABLE,
            Category::Gpu,
            "not-a-boolean",
            FactSource {
                collector: collector::GRAPHICS,
                method: ObservationMethod::Command("fixture".to_owned()),
            },
        )]);

        let outcomes = evaluate_all(&builder.build());
        let outcome = outcomes
            .iter()
            .find(|item| item.rule_id == rule::VULKAN_AVAILABLE)
            .unwrap();

        assert_eq!(outcome.status, RuleStatus::Skipped);
        assert!(outcome.candidates.is_empty());
    }

    #[test]
    fn powersave_governor_emits_high_severity_problem() {
        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::CPU_GOVERNOR,
            Category::System,
            "powersave",
            FactSource {
                collector: collector::SYSTEM,
                method: ObservationMethod::File(
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_owned(),
                ),
            },
        )]);

        let outcomes = evaluate_all(&builder.build());
        let outcome = outcomes
            .iter()
            .find(|item| item.rule_id == rule::CPU_GOVERNOR)
            .unwrap();

        assert_eq!(outcome.status, RuleStatus::Problem);
        assert_eq!(outcome.candidates.len(), 1);
        assert_eq!(
            outcome.candidates[0].id,
            crate::ids::problem::CPU_GOVERNOR_POWERSAVE
        );
    }

    #[test]
    fn schedutil_governor_passes_rule() {
        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::CPU_GOVERNOR,
            Category::System,
            "schedutil",
            FactSource {
                collector: collector::SYSTEM,
                method: ObservationMethod::File(
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_owned(),
                ),
            },
        )]);

        let outcomes = evaluate_all(&builder.build());
        let outcome = outcomes
            .iter()
            .find(|item| item.rule_id == rule::CPU_GOVERNOR)
            .unwrap();

        assert_eq!(outcome.status, RuleStatus::Passed);
    }

    #[test]
    fn missing_governor_fact_skips_cpu_governor_rule() {
        let outcomes = evaluate_all(&FactBuilder::new().build());
        let outcome = outcomes
            .iter()
            .find(|item| item.rule_id == rule::CPU_GOVERNOR)
            .unwrap();

        assert_eq!(outcome.status, RuleStatus::Skipped);
    }
}

//! Deterministic problem synthesis from rule outcomes only.

use std::collections::BTreeMap;

use crate::ids::{problem, ProblemId};
use crate::model::{Problem, Recommendation, Severity};
use crate::rules::RuleOutcome;

#[must_use]
pub fn synthesize(outcomes: &[RuleOutcome]) -> Vec<Problem> {
    let mut unique: BTreeMap<ProblemId, Problem> = BTreeMap::new();

    for candidate in outcomes.iter().flat_map(|outcome| &outcome.candidates) {
        unique
            .entry(candidate.id)
            .or_insert_with(|| finalize(candidate));
    }

    let mut problems: Vec<Problem> = unique.into_values().collect();
    problems.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then_with(|| left.category.cmp(&right.category))
            .then_with(|| left.id.cmp(&right.id))
    });
    problems
}

fn finalize(candidate: &crate::rules::ProblemCandidate) -> Problem {
    let severity = severity_for(&candidate.id);
    let recommendation = recommendation_for(&candidate.id);

    let summary = match candidate.id {
        problem::VULKAN_UNAVAILABLE => "Vulkan is not available",
        problem::STEAM_PROTON_MISSING => "No Proton runtime was detected",
        problem::STEAM_LIBRARY_UNREADABLE => "A Steam library could not be read",
        problem::GAMEMODE_DAEMON_UNAVAILABLE => "GameMode daemon is unavailable",
        _ => "A diagnostic rule reported a problem",
    };

    Problem {
        id: candidate.id,
        category: candidate.category,
        severity,
        summary: summary.to_owned(),
        details: candidate.evidence.clone(),
        recommendation: Some(recommendation),
    }
}

fn severity_for(id: &ProblemId) -> Severity {
    match *id {
        problem::VULKAN_UNAVAILABLE => Severity::Critical,
        problem::STEAM_PROTON_MISSING | problem::STEAM_LIBRARY_UNREADABLE => Severity::Medium,
        problem::GAMEMODE_DAEMON_UNAVAILABLE => Severity::Low,
        _ => {
            debug_assert!(false, "missing severity for ProblemId: {id:?}");
            Severity::Medium
        }
    }
}

fn recommendation_for(id: &ProblemId) -> Recommendation {
    match *id {
        problem::VULKAN_UNAVAILABLE => Recommendation {
            text: "Vulkan is not available.".to_owned(),
            platform_specific: BTreeMap::from([
                ("arch".to_owned(), "pacman -S vulkan-tools".to_owned()),
                ("fedora".to_owned(), "dnf install vulkan-tools".to_owned()),
                ("debian".to_owned(), "apt install vulkan-tools".to_owned()),
                ("ubuntu".to_owned(), "apt install vulkan-tools".to_owned()),
                (
                    "nixos".to_owned(),
                    "configuration.nix: environment.systemPackages = [ pkgs.vulkan-tools ];"
                        .to_owned(),
                ),
            ]),
            difficulty: 1,
            est_minutes: 1,
            requires_root: true,
            requires_reboot: false,
            docs_url: Some("https://wiki.archlinux.org/title/Vulkan".to_owned()),
        },
        problem::STEAM_PROTON_MISSING => Recommendation {
            text: "No Proton runtime was detected.".to_owned(),
            platform_specific: BTreeMap::from([
                ("arch".to_owned(), "pacman -S steam".to_owned()),
                (
                    "ubuntu".to_owned(),
                    "flatpak install flathub com.valvesoftware.Steam".to_owned(),
                ),
            ]),
            difficulty: 1,
            est_minutes: 1,
            requires_root: false,
            requires_reboot: false,
            docs_url: Some(
                "https://help.steampowered.com/en/faqs/view/0596-50CD-C0E4-1290".to_owned(),
            ),
        },
        problem::STEAM_LIBRARY_UNREADABLE => Recommendation {
            text: "A Steam library could not be read. Check filesystem permissions on the library mount.".to_owned(),
            platform_specific: BTreeMap::new(),
            difficulty: 2,
            est_minutes: 5,
            requires_root: true,
            requires_reboot: false,
            docs_url: None,
        },
        problem::GAMEMODE_DAEMON_UNAVAILABLE => Recommendation {
            text: "Feral GameMode daemon is not running. Start it with sudo systemctl enable --now gamemoded.".to_owned(),
            platform_specific: BTreeMap::new(),
            difficulty: 2,
            est_minutes: 2,
            requires_root: true,
            requires_reboot: false,
            docs_url: None,
        },
        _ => {
            debug_assert!(false, "missing recommendation for ProblemId: {id:?}");
            Recommendation {
                text: "Review the evidence in this report.".to_owned(),
                platform_specific: BTreeMap::new(),
                difficulty: 1,
                est_minutes: 1,
                requires_root: false,
                requires_reboot: false,
                docs_url: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::facts::Category;
    use crate::ids::{problem, rule};
    use crate::rules::{ProblemCandidate, RuleOutcome, RuleStatus};

    use super::synthesize;

    #[test]
    fn duplicate_problem_ids_are_collapsed() {
        let candidate = ProblemCandidate {
            id: problem::VULKAN_UNAVAILABLE,
            category: Category::Gpu,
            evidence: Vec::new(),
        };
        let outcome = RuleOutcome {
            rule_id: rule::VULKAN_AVAILABLE,
            status: RuleStatus::Problem,
            explanation: String::new(),
            candidates: vec![candidate.clone(), candidate],
        };

        assert_eq!(synthesize(&[outcome]).len(), 1);
    }

    #[test]
    fn every_registered_problem_has_a_valid_recommendation() {
        let problem_ids = [
            problem::VULKAN_UNAVAILABLE,
            problem::STEAM_PROTON_MISSING,
            problem::STEAM_LIBRARY_UNREADABLE,
            problem::GAMEMODE_DAEMON_UNAVAILABLE,
        ];
        for id in problem_ids {
            let rec = super::recommendation_for(&id);
            assert!(
                !rec.text.is_empty(),
                "Recommendation text cannot be empty for {id:?}"
            );
        }
    }
}

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
        problem::CPU_GOVERNOR_POWERSAVE => "CPU governor is set to powersave",
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
        problem::CPU_GOVERNOR_POWERSAVE => Severity::High,
        _ => {
            debug_assert!(false, "missing severity for ProblemId: {id:?}");
            Severity::Medium
        }
    }
}

fn recommendation_for(id: &ProblemId) -> Recommendation {
    match *id {
        problem::VULKAN_UNAVAILABLE => Recommendation {
            text: "GameDoc could not verify Vulkan support because vulkaninfo was not installed or did not report a working Vulkan device.".to_owned(),
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
            text: "Without a Proton runtime, Steam cannot launch Windows games through its Linux compatibility layer.".to_owned(),
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
            text: "GameDoc could not inspect this library, so installed games and compatibility tools may be missing from the report. Check the mount and grant your user read access.".to_owned(),
            platform_specific: BTreeMap::new(),
            difficulty: 2,
            est_minutes: 5,
            requires_root: true,
            requires_reboot: false,
            docs_url: None,
        },
        problem::GAMEMODE_DAEMON_UNAVAILABLE => Recommendation {
            text: "GameMode cannot apply its performance profile while the daemon is unavailable. Enable and start the service before launching a game.".to_owned(),
            platform_specific: BTreeMap::new(),
            difficulty: 2,
            est_minutes: 2,
            requires_root: true,
            requires_reboot: false,
            docs_url: None,
        },
        problem::CPU_GOVERNOR_POWERSAVE => Recommendation {
            text: "The 'powersave' governor keeps the CPU at its lowest clock speed even under heavy load. Switching to 'schedutil' or 'performance' allows the CPU to reach full speed and improves game performance.".to_owned(),
            platform_specific: BTreeMap::from([
                (
                    "arch".to_owned(),
                    "echo schedutil | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor".to_owned(),
                ),
                (
                    "fedora".to_owned(),
                    "echo schedutil | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor".to_owned(),
                ),
                (
                    "debian".to_owned(),
                    "echo schedutil | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor".to_owned(),
                ),
                (
                    "ubuntu".to_owned(),
                    "echo schedutil | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor".to_owned(),
                ),
                (
                    "nixos".to_owned(),
                    "powerManagement.cpuFreqGovernor = \"schedutil\"; # in configuration.nix".to_owned(),
                ),
            ]),
            difficulty: 1,
            est_minutes: 1,
            requires_root: true,
            requires_reboot: false,
            docs_url: Some("https://wiki.archlinux.org/title/CPU_frequency_scaling".to_owned()),
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
            problem::CPU_GOVERNOR_POWERSAVE,
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

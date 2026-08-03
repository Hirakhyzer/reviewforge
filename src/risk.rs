use std::collections::BTreeSet;

use crate::model::{ChangeKind, ChangedFile, FileRisk, Finding, Severity};

const CRITICAL_PATH_MARKERS: &[&str] = &[
    "auth", "security", "permission", "payment", "billing", "migration", "schema",
    "infra", "deploy", "docker", "workflow", "secret", "crypto",
];

const TEST_MARKERS: &[&str] = &["test", "tests", "spec", "__tests__"];
const SOURCE_EXTENSIONS: &[&str] = &["rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "kt", "swift", "c", "cpp"];

pub fn analyze(files: &[ChangedFile]) -> (Vec<FileRisk>, Vec<Finding>, u8, Vec<String>) {
    let mut risks = Vec::new();
    let mut findings = Vec::new();
    let mut checklist = BTreeSet::new();

    for file in files {
        let mut score: u16 = 5;
        let mut reasons = Vec::new();
        let path_lower = file.path.to_ascii_lowercase();

        let volume = file.total_changes();
        if volume >= 400 {
            score += 35;
            reasons.push(format!("large change set ({volume} lines)"));
        } else if volume >= 150 {
            score += 24;
            reasons.push(format!("substantial change set ({volume} lines)"));
        } else if volume >= 50 {
            score += 12;
            reasons.push(format!("moderate change set ({volume} lines)"));
        }

        if file.hunks >= 8 {
            score += 10;
            reasons.push(format!("changes spread across {} hunks", file.hunks));
        }

        if CRITICAL_PATH_MARKERS.iter().any(|marker| path_lower.contains(marker)) {
            score += 25;
            reasons.push("touches a security, deployment, data, or money-sensitive path".to_string());
            checklist.insert(format!("Validate critical-path behavior in `{}`", file.path));
        }

        if matches!(file.kind, ChangeKind::Deleted) {
            score += 15;
            reasons.push("deletes an entire file".to_string());
        }

        if is_dependency_file(&path_lower) {
            score += 15;
            reasons.push("changes dependency or lock metadata".to_string());
            checklist.insert("Review dependency provenance and version changes".to_string());
        }

        if is_ci_file(&path_lower) {
            score += 15;
            reasons.push("changes CI or release automation".to_string());
            checklist.insert("Verify CI permissions, triggers, and secret usage".to_string());
        }

        let risky_added = file.added_lines.iter().filter(|line| contains_risky_construct(line)).count();
        if risky_added > 0 {
            score += (risky_added.min(4) * 8) as u16;
            reasons.push(format!("contains {risky_added} potentially risky added line(s)"));
            findings.push(Finding {
                severity: if risky_added >= 3 { Severity::High } else { Severity::Medium },
                title: "Potentially risky construct added".to_string(),
                detail: "Review shell execution, unsafe blocks, broad permissions, disabled checks, or placeholder credentials introduced in this file.".to_string(),
                file: Some(file.path.clone()),
            });
        }

        if is_test_file(&path_lower) && matches!(file.kind, ChangeKind::Deleted) {
            findings.push(Finding {
                severity: Severity::High,
                title: "Test coverage removed".to_string(),
                detail: "A test file is deleted. Confirm equivalent coverage exists elsewhere.".to_string(),
                file: Some(file.path.clone()),
            });
        }

        risks.push(FileRisk {
            path: file.path.clone(),
            score: score.min(100) as u8,
            reasons,
        });
    }

    detect_test_gap(files, &mut findings, &mut checklist);
    detect_large_pr(files, &mut findings, &mut checklist);

    let overall = calculate_overall_score(&risks, &findings);
    if checklist.is_empty() {
        checklist.insert("Confirm the changed behavior matches the linked issue or requirement".to_string());
        checklist.insert("Run the relevant automated tests".to_string());
    }

    (risks, findings, overall, checklist.into_iter().collect())
}

pub fn label_for_score(score: u8) -> &'static str {
    match score {
        0..=24 => "low",
        25..=49 => "medium",
        50..=74 => "high",
        _ => "critical",
    }
}

fn calculate_overall_score(risks: &[FileRisk], findings: &[Finding]) -> u8 {
    if risks.is_empty() {
        return 0;
    }
    let max = risks.iter().map(|risk| risk.score).max().unwrap_or(0) as u16;
    let avg = risks.iter().map(|risk| u16::from(risk.score)).sum::<u16>() / risks.len() as u16;
    let finding_bonus: u16 = findings
        .iter()
        .map(|finding| match finding.severity {
            Severity::Low => 1,
            Severity::Medium => 4,
            Severity::High => 8,
            Severity::Critical => 15,
        })
        .sum();
    ((max * 2 + avg) / 3 + finding_bonus.min(20)).min(100) as u8
}

fn detect_test_gap(files: &[ChangedFile], findings: &mut Vec<Finding>, checklist: &mut BTreeSet<String>) {
    let source_changed = files.iter().any(|file| is_source_file(&file.path) && !is_test_file(&file.path));
    let tests_changed = files.iter().any(|file| is_test_file(&file.path));
    if source_changed && !tests_changed {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "No test changes detected".to_string(),
            detail: "Production source changed without an accompanying test-file change. Existing tests may still cover it, but reviewers should verify this explicitly.".to_string(),
            file: None,
        });
        checklist.insert("Verify existing tests cover the changed production behavior".to_string());
    }
}

fn detect_large_pr(files: &[ChangedFile], findings: &mut Vec<Finding>, checklist: &mut BTreeSet<String>) {
    let total: usize = files.iter().map(ChangedFile::total_changes).sum();
    if files.len() >= 20 || total >= 800 {
        findings.push(Finding {
            severity: Severity::High,
            title: "Large review surface".to_string(),
            detail: format!("This change spans {} files and {total} changed lines. Consider splitting it or assigning domain-specific reviewers.", files.len()),
            file: None,
        });
        checklist.insert("Consider splitting the pull request into smaller independently reviewable changes".to_string());
    }
}

fn contains_risky_construct(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "unsafe {",
        "shell=true",
        "chmod 777",
        "--no-verify",
        "allow-all",
        "permissions: write-all",
        "password = \"",
        "api_key = \"",
        "secret = \"",
        "todo: disable",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_dependency_file(path: &str) -> bool {
    ["cargo.toml", "cargo.lock", "package.json", "package-lock.json", "pnpm-lock", "yarn.lock", "requirements.txt", "poetry.lock", "go.mod", "go.sum"]
        .iter()
        .any(|name| path.ends_with(name))
}

fn is_ci_file(path: &str) -> bool {
    path.contains(".github/workflows/") || path.ends_with("gitlab-ci.yml") || path.contains("buildkite")
}

fn is_test_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    TEST_MARKERS.iter().any(|marker| lower.split(|c: char| matches!(c, '/' | '.' | '_' | '-')).any(|part| part == *marker))
}

fn is_source_file(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, ext)| SOURCE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, additions: usize, deletions: usize) -> ChangedFile {
        ChangedFile {
            old_path: Some(path.to_string()),
            path: path.to_string(),
            kind: ChangeKind::Modified,
            additions,
            deletions,
            hunks: 1,
            added_lines: Vec::new(),
            removed_lines: Vec::new(),
        }
    }

    #[test]
    fn flags_source_without_tests() {
        let (_, findings, _, _) = analyze(&[file("src/main.rs", 10, 2)]);
        assert!(findings.iter().any(|finding| finding.title == "No test changes detected"));
    }

    #[test]
    fn critical_path_scores_higher() {
        let (risks, _, _, _) = analyze(&[file("src/auth/session.rs", 10, 2)]);
        assert!(risks[0].score >= 30);
    }

    #[test]
    fn labels_scores() {
        assert_eq!(label_for_score(5), "low");
        assert_eq!(label_for_score(40), "medium");
        assert_eq!(label_for_score(60), "high");
        assert_eq!(label_for_score(90), "critical");
    }
}

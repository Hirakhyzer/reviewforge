#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    pub old_path: Option<String>,
    pub path: String,
    pub kind: ChangeKind,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: usize,
    pub added_lines: Vec<String>,
    pub removed_lines: Vec<String>,
}

impl ChangedFile {
    pub fn total_changes(&self) -> usize {
        self.additions + self.deletions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRisk {
    pub path: String,
    pub score: u8,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewReport {
    pub title: String,
    pub summary: String,
    pub issue_refs: Vec<String>,
    pub files: Vec<ChangedFile>,
    pub file_risks: Vec<FileRisk>,
    pub findings: Vec<Finding>,
    pub overall_score: u8,
    pub risk_label: String,
    pub checklist: Vec<String>,
}

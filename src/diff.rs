use crate::model::{ChangeKind, ChangedFile};

pub fn parse_unified_diff(input: &str) -> Result<Vec<ChangedFile>, String> {
    let mut files = Vec::new();
    let mut current: Option<ChangedFile> = None;

    for line in input.lines() {
        if let Some(header) = line.strip_prefix("diff --git a/") {
            if let Some(file) = current.take() {
                files.push(file);
            }
            let Some((old_path, new_part)) = header.split_once(" b/") else {
                return Err(format!("invalid diff header: {line}"));
            };
            current = Some(ChangedFile {
                old_path: Some(old_path.to_string()),
                path: new_part.to_string(),
                kind: ChangeKind::Modified,
                additions: 0,
                deletions: 0,
                hunks: 0,
                added_lines: Vec::new(),
                removed_lines: Vec::new(),
            });
            continue;
        }

        let Some(file) = current.as_mut() else {
            continue;
        };

        if line.starts_with("new file mode ") {
            file.kind = ChangeKind::Added;
        } else if line.starts_with("deleted file mode ") {
            file.kind = ChangeKind::Deleted;
        } else if let Some(path) = line.strip_prefix("rename from ") {
            file.old_path = Some(path.to_string());
            file.kind = ChangeKind::Renamed;
        } else if let Some(path) = line.strip_prefix("rename to ") {
            file.path = path.to_string();
            file.kind = ChangeKind::Renamed;
        } else if line.starts_with("@@") {
            file.hunks += 1;
        } else if line.starts_with("+++") || line.starts_with("---") {
            continue;
        } else if let Some(added) = line.strip_prefix('+') {
            file.additions += 1;
            file.added_lines.push(added.to_string());
        } else if let Some(removed) = line.strip_prefix('-') {
            file.deletions += 1;
            file.removed_lines.push(removed.to_string());
        }
    }

    if let Some(file) = current {
        files.push(file);
    }

    if files.is_empty() && !input.trim().is_empty() {
        return Err("no `diff --git` file headers found".to_string());
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_added_and_removed_lines() {
        let diff = "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1 +1,2 @@\n-old\n+new\n+line\n";
        let files = parse_unified_diff(diff).expect("parse");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].additions, 2);
        assert_eq!(files[0].deletions, 1);
        assert_eq!(files[0].hunks, 1);
    }

    #[test]
    fn parses_new_file() {
        let diff = "diff --git a/new.rs b/new.rs\nnew file mode 100644\n--- /dev/null\n+++ b/new.rs\n@@ -0,0 +1 @@\n+hello\n";
        let files = parse_unified_diff(diff).expect("parse");
        assert_eq!(files[0].kind, ChangeKind::Added);
    }

    #[test]
    fn rejects_non_diff_text() {
        assert!(parse_unified_diff("hello").is_err());
    }
}

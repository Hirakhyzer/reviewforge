use std::fmt::Write as _;

use crate::model::{ChangedFile, ReviewReport};

pub fn markdown(report: &ReviewReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", report.title);
    let _ = writeln!(out, "\n> **Risk:** {} / 100 ({})", report.overall_score, report.risk_label);
    let _ = writeln!(out, "\n{}", report.summary);

    if !report.issue_refs.is_empty() {
        let _ = writeln!(out, "\n## Linked issues\n\n{}", report.issue_refs.join(", "));
    }

    let _ = writeln!(out, "\n## Change overview\n");
    let _ = writeln!(out, "| File | Type | + | - | Risk |");
    let _ = writeln!(out, "|---|---:|---:|---:|---:|");
    for file in &report.files {
        let score = report.file_risks.iter().find(|risk| risk.path == file.path).map_or(0, |risk| risk.score);
        let _ = writeln!(out, "| `{}` | {} | {} | {} | {} |", file.path, file.kind.as_str(), file.additions, file.deletions, score);
    }

    let _ = writeln!(out, "\n## Findings\n");
    if report.findings.is_empty() {
        let _ = writeln!(out, "No automated review findings.");
    } else {
        for finding in &report.findings {
            let location = finding.file.as_deref().map_or(String::new(), |file| format!(" — `{file}`"));
            let _ = writeln!(out, "- **{}:** {}{} — {}", finding.severity.as_str(), finding.title, location, finding.detail);
        }
    }

    let _ = writeln!(out, "\n## Reviewer checklist\n");
    for item in &report.checklist {
        let _ = writeln!(out, "- [ ] {item}");
    }

    let _ = writeln!(out, "\n## Per-file risk reasons\n");
    for risk in &report.file_risks {
        if risk.reasons.is_empty() {
            continue;
        }
        let _ = writeln!(out, "### `{}` — {}/100", risk.path, risk.score);
        for reason in &risk.reasons {
            let _ = writeln!(out, "- {reason}");
        }
    }

    out
}

pub fn json(report: &ReviewReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    json_field(&mut out, "title", &report.title, true, 2);
    json_field(&mut out, "summary", &report.summary, true, 2);
    let _ = writeln!(out, "  \"overall_score\": {},", report.overall_score);
    json_field(&mut out, "risk_label", &report.risk_label, true, 2);
    out.push_str("  \"issue_refs\": [");
    for (index, issue) in report.issue_refs.iter().enumerate() {
        if index > 0 { out.push_str(", "); }
        let _ = write!(out, "\"{}\"", escape_json(issue));
    }
    out.push_str("],\n  \"files\": [\n");
    for (index, file) in report.files.iter().enumerate() {
        let comma = if index + 1 == report.files.len() { "" } else { "," };
        let _ = writeln!(out, "    {{\"path\": \"{}\", \"kind\": \"{}\", \"additions\": {}, \"deletions\": {}, \"hunks\": {}}}{comma}", escape_json(&file.path), file.kind.as_str(), file.additions, file.deletions, file.hunks);
    }
    out.push_str("  ],\n  \"findings\": [\n");
    for (index, finding) in report.findings.iter().enumerate() {
        let comma = if index + 1 == report.findings.len() { "" } else { "," };
        let file = finding.file.as_deref().map_or("null".to_string(), |value| format!("\"{}\"", escape_json(value)));
        let _ = writeln!(out, "    {{\"severity\": \"{}\", \"title\": \"{}\", \"detail\": \"{}\", \"file\": {file}}}{comma}", finding.severity.as_str(), escape_json(&finding.title), escape_json(&finding.detail));
    }
    out.push_str("  ],\n  \"checklist\": [");
    for (index, item) in report.checklist.iter().enumerate() {
        if index > 0 { out.push_str(", "); }
        let _ = write!(out, "\"{}\"", escape_json(item));
    }
    out.push_str("]\n}\n");
    out
}

pub fn html(report: &ReviewReport) -> String {
    let mut rows = String::new();
    for file in &report.files {
        let score = report.file_risks.iter().find(|risk| risk.path == file.path).map_or(0, |risk| risk.score);
        let _ = write!(rows, "<tr><td><code>{}</code></td><td>{}</td><td>+{}</td><td>-{}</td><td><strong>{}</strong></td></tr>", escape_html(&file.path), file.kind.as_str(), file.additions, file.deletions, score);
    }
    let findings = if report.findings.is_empty() {
        "<p>No automated review findings.</p>".to_string()
    } else {
        report.findings.iter().map(|finding| {
            let location = finding.file.as_deref().map_or(String::new(), |file| format!(" <code>{}</code>", escape_html(file)));
            format!("<article class=\"finding {}\"><strong>{}: {}</strong>{}<p>{}</p></article>", finding.severity.as_str(), finding.severity.as_str(), escape_html(&finding.title), location, escape_html(&finding.detail))
        }).collect::<Vec<_>>().join("\n")
    };
    let checklist = report.checklist.iter().map(|item| format!("<li><input type=\"checkbox\"> {}</li>", escape_html(item))).collect::<Vec<_>>().join("\n");
    let issues = if report.issue_refs.is_empty() { "None detected".to_string() } else { report.issue_refs.iter().map(|item| format!("<code>{}</code>", escape_html(item))).collect::<Vec<_>>().join(" ") };

    format!(r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title><style>
:root{{--bg:#0b1020;--panel:#141b2d;--text:#edf2ff;--muted:#9aa7c2;--line:#2c3650;--accent:#89b4ff}}
*{{box-sizing:border-box}}body{{margin:0;font:15px/1.55 system-ui;background:var(--bg);color:var(--text)}}main{{max-width:1050px;margin:auto;padding:40px 20px}}.hero,.panel{{background:var(--panel);border:1px solid var(--line);border-radius:16px;padding:24px;margin-bottom:18px}}h1{{margin:0 0 8px;font-size:34px}}h2{{margin-top:0}}.score{{font-size:44px;font-weight:800;color:var(--accent)}}.muted{{color:var(--muted)}}table{{width:100%;border-collapse:collapse}}th,td{{padding:10px;border-bottom:1px solid var(--line);text-align:left}}code{{background:#0d1425;padding:2px 6px;border-radius:6px}}.finding{{padding:12px 14px;border-left:4px solid var(--accent);background:#0d1425;margin:10px 0;border-radius:8px}}.finding.high,.finding.critical{{border-left-color:#ff7b72}}.finding.medium{{border-left-color:#f2cc60}}li{{margin:8px 0}}input{{accent-color:var(--accent)}}
</style></head><body><main>
<section class="hero"><div class="muted">ReviewForge report</div><h1>{title}</h1><div class="score">{score}/100</div><strong>{label} risk</strong><p>{summary}</p><p class="muted">Issue references: {issues}</p></section>
<section class="panel"><h2>Change overview</h2><table><thead><tr><th>File</th><th>Type</th><th>Add</th><th>Delete</th><th>Risk</th></tr></thead><tbody>{rows}</tbody></table></section>
<section class="panel"><h2>Findings</h2>{findings}</section>
<section class="panel"><h2>Reviewer checklist</h2><ul>{checklist}</ul></section>
</main></body></html>"#,
        title = escape_html(&report.title), score = report.overall_score, label = escape_html(&report.risk_label), summary = escape_html(&report.summary), issues = issues, rows = rows, findings = findings, checklist = checklist)
}

pub fn summarize_files(files: &[ChangedFile]) -> String {
    let additions: usize = files.iter().map(|file| file.additions).sum();
    let deletions: usize = files.iter().map(|file| file.deletions).sum();
    format!("{} files changed with {additions} additions and {deletions} deletions.", files.len())
}

fn json_field(out: &mut String, key: &str, value: &str, comma: bool, indent: usize) {
    let comma = if comma { "," } else { "" };
    let _ = writeln!(out, "{}\"{}\": \"{}\"{comma}", " ".repeat(indent), key, escape_json(value));
}

fn escape_json(input: &str) -> String {
    input.chars().flat_map(|c| match c {
        '\\' => "\\\\".chars().collect::<Vec<_>>(),
        '"' => "\\\"".chars().collect(),
        '\n' => "\\n".chars().collect(),
        '\r' => "\\r".chars().collect(),
        '\t' => "\\t".chars().collect(),
        other => vec![other],
    }).collect()
}

fn escape_html(input: &str) -> String {
    input.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChangeKind, FileRisk, ReviewReport};

    fn report() -> ReviewReport {
        ReviewReport {
            title: "Test".into(), summary: "Summary".into(), issue_refs: vec!["#1".into()],
            files: vec![ChangedFile { old_path: None, path: "src/main.rs".into(), kind: ChangeKind::Added, additions: 1, deletions: 0, hunks: 1, added_lines: vec![], removed_lines: vec![] }],
            file_risks: vec![FileRisk { path: "src/main.rs".into(), score: 20, reasons: vec![] }], findings: vec![], overall_score: 20, risk_label: "low".into(), checklist: vec!["Check".into()]
        }
    }

    #[test]
    fn emits_valid_shapes() {
        assert!(markdown(&report()).contains("# Test"));
        assert!(json(&report()).contains("\"overall_score\": 20"));
        assert!(html(&report()).contains("<!doctype html>"));
    }
}

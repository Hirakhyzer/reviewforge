mod diff;
mod model;
mod report;
mod risk;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use model::ReviewReport;

#[derive(Debug, Clone, Copy)]
enum Format {
    Markdown,
    Json,
    Html,
}

struct Args {
    diff_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    format: Format,
    title: String,
    context: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("reviewforge: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1).collect())?;
    let input = read_input(args.diff_path.as_ref())?;
    let files = diff::parse_unified_diff(&input)?;
    let (file_risks, findings, overall_score, checklist) = risk::analyze(&files);
    let issue_refs = extract_issue_refs(&args.context);
    let summary = report::summarize_files(&files);
    let report = ReviewReport {
        title: args.title,
        summary,
        issue_refs,
        files,
        file_risks,
        findings,
        overall_score,
        risk_label: risk::label_for_score(overall_score).to_string(),
        checklist,
    };

    let rendered = match args.format {
        Format::Markdown => report::markdown(&report),
        Format::Json => report::json(&report),
        Format::Html => report::html(&report),
    };

    if let Some(path) = args.output_path {
        fs::write(&path, rendered).map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        println!("Review report written to {}", path.display());
    } else {
        print!("{rendered}");
    }
    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        std::process::exit(0);
    }
    if args.iter().any(|arg| arg == "--version") {
        println!("reviewforge {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let mut parsed = Args {
        diff_path: None,
        output_path: None,
        format: Format::Markdown,
        title: "Code Review Report".to_string(),
        context: String::new(),
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "analyze" => {}
            "--diff" | "-d" => {
                index += 1;
                parsed.diff_path = Some(PathBuf::from(args.get(index).ok_or("--diff requires a path")?));
            }
            "--output" | "-o" => {
                index += 1;
                parsed.output_path = Some(PathBuf::from(args.get(index).ok_or("--output requires a path")?));
            }
            "--format" | "-f" => {
                index += 1;
                parsed.format = match args.get(index).map(String::as_str) {
                    Some("markdown" | "md") => Format::Markdown,
                    Some("json") => Format::Json,
                    Some("html") => Format::Html,
                    Some(other) => return Err(format!("unsupported format `{other}`")),
                    None => return Err("--format requires markdown, json, or html".to_string()),
                };
            }
            "--title" => {
                index += 1;
                parsed.title = args.get(index).ok_or("--title requires text")?.clone();
            }
            "--context" => {
                index += 1;
                parsed.context = args.get(index).ok_or("--context requires text")?.clone();
            }
            other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
            other => {
                if parsed.diff_path.is_none() {
                    parsed.diff_path = Some(PathBuf::from(other));
                } else {
                    return Err(format!("unexpected argument `{other}`"));
                }
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn read_input(path: Option<&PathBuf>) -> Result<String, String> {
    match path {
        Some(path) => fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display())),
        None => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).map_err(|error| format!("cannot read stdin: {error}"))?;
            if input.trim().is_empty() {
                Err("provide a unified diff with --diff <file> or through stdin".to_string())
            } else {
                Ok(input)
            }
        }
    }
}

fn extract_issue_refs(context: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for token in context.split(|c: char| c.is_whitespace() || ",;()[]{}".contains(c)) {
        let candidate = token.trim_matches(|c: char| ".:!?".contains(c));
        if let Some(number) = candidate.strip_prefix('#') {
            if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) {
                let value = format!("#{number}");
                if !refs.contains(&value) { refs.push(value); }
            }
        }
    }
    refs
}

fn print_help() {
    println!("ReviewForge — offline-first code review report generator\n\nUSAGE:\n  reviewforge analyze --diff changes.diff [OPTIONS]\n  git diff | reviewforge analyze [OPTIONS]\n\nOPTIONS:\n  -d, --diff <PATH>       Unified diff file; stdin when omitted\n  -f, --format <FORMAT>   markdown, json, or html [default: markdown]\n  -o, --output <PATH>     Write report to a file\n      --title <TEXT>       Report title\n      --context <TEXT>     PR title/body text used to detect issue refs\n  -h, --help              Show help\n      --version           Show version");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_issue_references() {
        assert_eq!(extract_issue_refs("Fixes #12 and refs #12, #44."), vec!["#12", "#44"]);
    }

    #[test]
    fn parses_format() {
        let args = parse_args(vec!["analyze".into(), "--format".into(), "json".into()]).expect("args");
        assert!(matches!(args.format, Format::Json));
    }
}

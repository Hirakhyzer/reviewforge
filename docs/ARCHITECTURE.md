# Architecture

ReviewForge is intentionally split into four layers:

1. `diff`: parses standard unified Git patches into changed-file models.
2. `risk`: applies deterministic, explainable heuristics.
3. `report`: renders Markdown, JSON, and self-contained HTML.
4. `main`: CLI argument handling and input/output orchestration.

## Design principles

- **Offline first:** source code is not uploaded.
- **Explainable:** every score has reasons.
- **Deterministic:** the same input produces the same report.
- **Fail clearly:** malformed or unsupported input returns an error.
- **Extensible:** future rule packs can build on the same file model.

## Trust boundary

ReviewForge does not execute code from a diff. It treats each input line as text. The generated HTML escapes paths, titles, summaries, findings, and checklist content before rendering.

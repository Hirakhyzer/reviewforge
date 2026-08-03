# Contributing

Thank you for improving ReviewForge.

## Setup

```bash
git clone https://github.com/Hirakhyzer/reviewforge.git
cd reviewforge
cargo test
```

## Contribution workflow

1. Create a focused branch from `main`.
2. Add or change one explainable behavior.
3. Add a regression test.
4. Run formatting, Clippy, and tests.
5. Open a pull request describing the signal, false-positive risk, and validation.

## Rule quality

Risk rules should be conservative. A finding must state what a reviewer should verify; it must not claim a vulnerability has been proven.

## Commit messages

Use Conventional Commits, for example:

```text
feat(risk): detect deleted migration rollback
fix(diff): preserve renamed file paths
```

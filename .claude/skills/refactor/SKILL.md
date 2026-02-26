---
name: refactor
description: Apply safe, incremental Tidy First refactoring to target code. Use when the user asks to clean up, improve, or restructure existing code without changing behavior.
disable-model-invocation: true
argument-hint: [file, module, or crate to refactor]
---

Analyze and refactor the following target — make no behavioral changes, only structural improvements: $ARGUMENTS

## Step 1 — Verify Green Baseline

Run `cargo test`. If any test fails, stop and report it. Refactoring only begins when all tests pass.

## Step 2 — Identify Refactoring Targets

Scan the target code for these smells, in priority order:

**Naming problems**
- Names that describe *how* code works or its history instead of *what* it does.
- Fix: `MCPWrapper` → `RemoteTool`, `NewAPI` → actual domain name, `executeToolWithValidation` → `execute`.
- Rule: names must tell a domain story. Never use implementation details, pattern names without clarity, or temporal context (`New`, `Legacy`, `Improved`, `Enhanced`, `Unified`).

**Structural problems**
- Functions doing more than one thing — extract and name by single responsibility.
- Deep nesting — flatten with early returns.
- Explicit `match`/`if let` on `Option`/`Result` where combinators are clearer (`map`, `and_then`, `unwrap_or_else`, `ok_or`).

**Duplication**
- Repeated logic across functions or modules — extract to one authoritative location.

**Coupling and state**
- Implicit dependencies — make them explicit via parameters or trait bounds.
- Unnecessary mutable state or side effects — minimize both.

## Step 3 — Apply One Refactoring at a Time

For each target, name the pattern and make only that change:

- **Extract Function** — pull a block into a named function.
- **Rename** — rename variable, function, type, or module to express domain intent.
- **Inline** — remove an unnecessary indirection.
- **Move** — relocate code to the crate or module where it belongs.
- **Replace Match with Combinator** — use `map`, `and_then`, `ok_or`, etc.
- **Introduce Early Return** — reduce nesting by propagating errors or returning eagerly.
- **Remove Duplication** — unify repeated logic into a single location.

After each individual change, run:

```
cargo test
```

If tests go red, revert that change immediately and investigate before continuing.

## Step 4 — Final Quality Gate

```
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Then run the pre-commit hook and fix any errors it raises.

## Step 5 — Commit Each Structural Change Separately

- Use `refactor:` prefix (Conventional Commits).
- Each commit must be **purely structural** — no behavior changes mixed in.
- Prefer small, frequent commits: `refactor: extract chunk_text into embeddings module`.

## Done When

- All tests remain green throughout
- No behavior has changed — only structure
- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` are all clean
- Pre-commit hook passes
- Every change is committed with a `refactor:` Conventional Commits message

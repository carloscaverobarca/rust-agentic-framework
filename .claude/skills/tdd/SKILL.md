---
name: tdd
description: Apply Red → Green → Refactor TDD cycle to implement a feature or fix. Use when the user asks to implement something new, fix a bug, or add behavior using TDD.
disable-model-invocation: true
argument-hint: [feature or behavior to implement]
---

Apply the Red → Green → Refactor TDD cycle to implement: $ARGUMENTS

Work through each phase sequentially. Commit at the end of each phase.

## Phase 1 — Red: Write a Failing Test

1. Identify the smallest unit of behavior to add or fix.
2. Write a descriptive, behavior-focused test name: `should_<expected_behavior>` (e.g., `should_return_error_on_invalid_input`, `should_stream_assistant_output_events`).
3. Place unit tests colocated with the code under test in a `#[cfg(test)]` module in the same file.
4. Place cross-crate tests under `crates/server/tests/`.
5. Prefer integration tests over mocks. Only mock at true external boundaries (network, AWS Bedrock, database).
6. Run `cargo test` and confirm the test **fails for the expected reason** — not a compile error unrelated to the missing behavior.
7. Commit: `test: <behavior being specified>`

## Phase 2 — Green: Make It Pass Minimally

1. Write the **minimum** production code required to make the failing test pass — nothing more.
2. Do not add logic not yet required by a failing test.
3. Use `?` for error propagation. No `unwrap`/`expect` outside tests.
4. Prefer functional combinators (`map`, `and_then`, `unwrap_or_else`) over explicit `match`/`if let` on `Option`/`Result`.
5. Run `cargo test` — all tests must pass.
6. Commit: `feat: <behavior implemented>` or `fix: <bug fixed>`

## Phase 3 — Refactor: Improve Structure Without Changing Behavior

Only after all tests are green:

1. Apply **one refactoring at a time**: extract function, rename, remove duplication, replace `match` with combinator, introduce early return, move to correct module.
2. Name things by what they **do** in the domain — never by implementation detail, pattern, or history:
   - Good: `Tool`, `Registry`, `execute()`
   - Bad: `MCPWrapper`, `NewHandler`, `executeToolWithValidation()`
3. Run `cargo test` after **each** individual change. If tests go red, revert and investigate.
4. Commit each structural change separately: `refactor: <what was improved>`

## Quality Gates (before every commit)

```
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Then run the pre-commit hook and fix any errors it raises.

## Done When

- All tests pass
- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` are clean
- Pre-commit hook passes
- Each phase has its own Conventional Commits commit(s)
- No structural and behavioral changes are mixed in a single commit

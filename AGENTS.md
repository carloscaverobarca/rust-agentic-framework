# AGENTS.md

## Communication & Workflow Rules

- **Always clarify ambiguities**: NEVER assume requirements. Ask for clarification if anything is unclear.
- **Error Handling**: Use proper error types and propagation. Prefer combinators (`map`, `and_then`, `unwrap_or`) over pattern matching for `Option`/`Result`.
- **Documentation**: Document public APIs and complex logic. Keep docs up to date with code changes.
- **Testing Discipline**: Place unit tests with the code they test. Use integration tests for cross-crate behavior. Run all tests, lints, and formatters before merging.
- **Feedback**: Give honest, constructive feedback. Do not agree just to be agreeable—technical judgment is valued.
- **No Unneeded Comments**: Code should be self-explanatory. Remove redundant comments.

## Agent/AI Collaboration

- **Explicit Instructions**: When requesting work, specify the desired outcome, acceptance criteria, and any constraints.
- **Status Updates**: For multi-step or nontrivial tasks, maintain a visible todo list and update progress as work advances.
- **No Sycophancy**: Avoid empty praise. Focus on actionable, technical feedback.
- **Pre-commit Checks**: Always run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before committing.
- **Rust Style**: Use functional style, early returns, and explicit types on public APIs. Avoid `unwrap`/`expect` outside tests.

**Development cycle (TDD — Red → Green → Refactor):**
1. Write one small failing test.
2. Implement the minimum to pass it.
3. Refactor only once tests are green.

**Tidy First — never mix in one commit:**
- STRUCTURAL: renames, extractions, moves (no behavior change).
- BEHAVIORAL: new or changed functionality.
- Order: structural first → run tests → commit → then behavioral.

**Commit rules:**
- All tests pass and no linter warnings before committing.
- Use Conventional Commits: `feat:`, `fix:`, `refactor:`, `test:`, `build:`.
- One logical unit per commit; label it structural or behavioral.

**Code quality:**
- Small, focused methods; clear naming; no duplication.
- Run tests after every refactor step.
- **YAGNI (You Aren't Gonna Need It):** Only implement what is required right now. Never add functionality, abstractions, or flexibility on the assumption that it will be needed later.

**XP Principles:**
- **Communication:** Share knowledge openly; no siloed understanding of the codebase.
- **Simplicity:** Implement the simplest thing that works; avoid speculative complexity.
- **Feedback:** Tests and CI are the primary feedback loop; act on failures immediately.
- **Courage:** Refactor without fear when tests are green; delete dead code without hesitation.
- **Respect:** Leave code cleaner than you found it; every change should add clear value.
- **Small increments:** Deliver one behaviour at a time; each commit should be a shippable step forward.

---

These rules are designed to ensure efficient, high-quality, and transparent collaboration between human and AI agents in this repository.

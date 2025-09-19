# AGENTS.md

## Communication & Workflow Rules

- **Always clarify ambiguities**: NEVER assume requirements. Ask for clarification if anything is unclear.
- **TDD First**: Start every feature or fix with a failing test. Use descriptive, behavior-focused test names (e.g., `should_return_error_on_invalid_input`).
- **Tidy First**: Separate structural (refactoring, renaming, moving code) from behavioral (new features, bug fixes) changes. Never mix both in a single commit.
- **Small, Frequent Commits**: Commit only when all tests pass and the change is a single logical unit. Use clear commit messages indicating structural or behavioral intent.
- **Code Quality**: Prioritize readability and maintainability over cleverness or premature optimization. Eliminate duplication and keep functions small and focused.
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

---

These rules are designed to ensure efficient, high-quality, and transparent collaboration between human and AI agents in this repository.

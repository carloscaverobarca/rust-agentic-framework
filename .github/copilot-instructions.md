## Development Philosophy

- Test-Driven Development (TDD) with "Red → Green → Refactor" cycle
- "Tidy First" approach separating structural from behavioral changes

## Coding Standards

### Rust Style Guidelines
- Prefer functional programming style over imperative
- Use Option/Result combinators (map, and_then, unwrap_or) over pattern matching
- Use `?` for error propagation
- Avoid `unwrap`/`expect` outside tests
- Keep functions small with explicit types on public APIs
- Use early returns to avoid deep nesting

### Testing Approach
1. Write failing test first (Red)
2. Implement minimum code to pass (Green)
3. Refactor while keeping tests green
4. Follow test naming convention: `should_do_something_when_condition`
5. Use descriptive test names that specify behavior

### Code Organization
- Keep related code together in appropriate crates
- Follow standard Rust project layout
- Use meaningful module hierarchies
- Place unit tests in the same file as the code they test
- Put integration tests in `crates/server/tests/`

### Error Handling
- Use proper error types and propagation
- Implement custom error types when needed
- Follow the error handling strategy defined in the project
- Provide meaningful error messages

### Documentation
- Document public APIs
- Include examples in doc comments
- Explain complex algorithms or business logic
- Keep documentation up to date with code changes

## Project Structure

```
agentic-framework/
├── crates/
│   ├── agentic-core/       # shared types, config, session management
│   ├── server/             # Axum HTTP & SSE server (main binary)
│   ├── embeddings/         # Cohere client with fallback, text chunker
│   ├── vector_store/       # pgvector integration
│   ├── llm/                # AWS Bedrock Claude wrapper with streaming
│   └── tooling/            # file_summarizer tool + tool registry
```

## Common Patterns

### Error Handling
```rust
// Prefer this:
fn process_data() -> Result<Data, Error> {
    let result = do_something()?;
    Ok(result)
}

// Over this:
fn process_data() -> Result<Data, Error> {
    match do_something() {
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    }
}
```

### Option Handling
```rust
// Prefer this:
fn get_user_data(id: &str) -> Option<UserData> {
    some_map.get(id)
        .map(|data| data.into_user_data())
}

// Over this:
fn get_user_data(id: &str) -> Option<UserData> {
    if let Some(data) = some_map.get(id) {
        Some(data.into_user_data())
    } else {
        None
    }
}
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_process_valid_input() {
        let input = "valid data";
        let result = process_input(input);
        assert!(result.is_ok());
    }
}
```

## Key Dependencies

Be familiar with these core dependencies when suggesting code:
- **axum** (0.7) - Web framework for HTTP and SSE
- **tokio** (1.0) - Async runtime
- **sqlx** (0.8) - Database toolkit
- **pgvector** (0.4) - Vector store
- **serde** (1.0) - Serialization
- **aws-sdk-bedrockruntime** (1.15) - AWS Bedrock

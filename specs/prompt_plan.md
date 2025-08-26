# 🗂️ Prompt Plan for Agentic Chatbot Backend (Rust)

This document provides:

1. **Detailed Blueprint** – a step‑by‑step implementation plan derived from `agentic_chatbot_spec.md`.
2. **Incremental Roadmap** – phases → chunks → atomic steps.
3. **LLM Code‑Generation Prompts** – ready‑to‑copy prompts (tagged as `text`) for a code‑generation model to implement each step in a test‑driven workflow.

---

## 1. Implementation Blueprint

### 1.1 Repository Layout
```
agentic_chatbot/
├── Cargo.toml              # workspace root
├── crates/
│   ├── core/               # shared types, config, errors
│   ├── server/             # Axum HTTP & SSE
│   ├── embeddings/         # Cohere client, text chunker
│   ├── vector_store/       # pgvector integration
│   ├── llm/                # Bedrock Claude wrapper
│   └── tooling/            # file_summarizer + registry
└── tests/                  # integration tests
```

### 1.2 Module Responsibilities
| Crate / Module | Purpose |
|----------------|---------|
| **core** | Config loader, message types, session store, custom errors |
| **embeddings** | Document chunking, Cohere embeddings + fallback |
| **vector_store** | pgvector schema & similarity search |
| **llm** | Bedrock Claude Sonnet v4 wrapper + 3.7 fallback |
| **tooling** | `Tool` trait, registry, `file_summarizer` |
| **server** | Axum routes, SSE streaming, agent loop |

### 1.3 Data Flow
1. Client sends request to `/predict_stream`.
2. Server loads session history, detects tool usage.
3. Executes `file_summarizer` if needed → emits `tool_usage` SSE.
4. Retrieves relevant chunks from `vector_store`.
5. Calls `llm` with prompt + context; streams `assistant_output`.
6. Updates session history.

### 1.4 Error Handling
| Layer | Error | Strategy |
|-------|-------|----------|
| Tool | FileNotFound | Send `tool_usage` event with `"error": "FileNotFound"` |
| Embedding | Cohere 5xx | Exponential backoff ×3 then 500 |
| LLM | Timeout | Retry once with fallback model |
| SSE | Disconnect | Close stream gracefully |
| DB | Connection lost | Reconnect once then 500 |

### 1.5 Testing
- **Unit**: Per‑crate logic.
- **Integration**: End‑to‑end chat with dockerised Postgres & mocked external APIs.
- **CI**: GitHub Actions – `fmt`, `clippy`, tests.

---

## 2. Incremental Roadmap

| Phase | Goal | Chunks |
|-------|------|--------|
| 0 | Foundation | 0‑A Init workspace • 0‑B CI |
| 1 | Core Types & Config | 1‑A Message enum • 1‑B Config loader • 1‑C SessionStore |
| 2 | Server Skeleton | 2‑A Axum setup • 2‑B SSE utils • 2‑C Stub endpoint |
| 3 | Embeddings | 3‑A Chunker • 3‑B Cohere client • 3‑C Fallback stub |
| 4 | Vector Store | 4‑A Migration • 4‑B Insert • 4‑C Query |
| 5 | LLM Wrapper | 5‑A Credentials • 5‑B Sonnet v4 call • 5‑C Fallback retry |
| 6 | Tooling | 6‑A Trait & registry • 6‑B file_summarizer • 6‑C Keyword detector |
| 7 | Agent & SSE | 7‑A Agent loop • 7‑B tool_usage event • 7‑C assistant_output stream |
| 8 | Polish & Tests | 8‑A Error types • 8‑B Integration tests • 8‑C Docs & sample config |

---

## 3. Atomic Step Tree

Example for **Phase 2 – Server Skeleton**:

| ID | Step |
|----|------|
| 2‑A‑1 | Add `axum = "0.7"` & `tokio` deps to `server/Cargo.toml`. |
| 2‑A‑2 | Create `server/src/main.rs` with `#[tokio::main]` + `Router::new().route("/health", get(|| async {"ok"}))`. |
| 2‑A‑3 | Add unit test hitting `/health` using `tower::ServiceExt`. |
| … | … |

*(The full atomic list accompanies each prompt section below.)*

---

## 4. LLM Code‑Generation Prompts

> Feed prompts **sequentially**.  
> After each generated diff, run tests; proceed only when green.

### Prompt Formatting
```text
🔖 <ID>
<Instruction>
```

### Phase 0 – Foundation

```text
🔖 0‑A‑1
Create a new Cargo **workspace** named `agentic_chatbot`.  
Add a root `Cargo.toml` with `[workspace] members = ["crates/*"]` and `.gitignore` (Rust + VSCode).  
Do NOT create any crates yet.  
Return the complete file list and contents.
```

```text
🔖 0‑A‑2
Inside `crates`, scaffold two crates:  
1. `core` (library) with empty `lib.rs`.  
2. `server` (binary) with `main.rs` printing "stub".  
Update workspace `Cargo.toml`.  
Ensure `cargo check` passes.
```

```text
🔖 0‑B‑1
Add GitHub Actions workflow `.github/workflows/ci.yml` that on `push`/`pull_request`:  
- Sets up Rust stable.  
- Runs `cargo fmt -- --check`, `cargo clippy --all -- -D warnings`, `cargo test`.  
Provide full YAML.
```

### Phase 1 – Core Types & Config

```text
🔖 1‑A‑1
In `core`, define enum `Role { User, Assistant, Tool }` and struct `Message { role: Role, content: String, name: Option<String> }` with serde derive + unit tests for serialization.
```

```text
🔖 1‑B‑1
Implement `Config` struct in `core` with fields mirroring `config.toml`.  
Add `config::load()` that reads `CONFIG_PATH` env (default `./config.toml`).  
Unit test with a temp file.
```

```text
🔖 1‑C‑1
Add `SessionStore` struct (HashMap<Uuid, Vec<Message>>) with methods `get`, `append`, `gc(ttl_secs)`.  
Cover with unit tests using `tokio::time::pause`.
```

*(Continue prompts for all phases through 8‑C‑3. Each atomic prompt is < ~150 tokens, ensuring small, testable steps.)*

---

**End of prompt_plan.md**
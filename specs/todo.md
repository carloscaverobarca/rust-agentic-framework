# ✅ TODO Checklist — Agentic Chatbot Backend MVP

Mark each task with **[x]** once complete.

---

## Phase 0 – Foundation
- [X] **0‑A Initialise workspace**
  - [X] 0‑A‑1 Create Cargo workspace `agentic_chatbot` with root `Cargo.toml` (`[workspace]`)
  - [X] 0‑A‑2 Add `.gitignore` (Rust + VSCode)
  - [X] 0‑A‑3 Scaffold empty `crates/` dir
- [X] **0‑B Configure CI**
  - [X] 0‑B‑1 Add GitHub Actions workflow (`fmt`, `clippy`, `test`)
  - [X] 0‑B‑2 Ensure CI passes on empty workspace commit

---

## Phase 1 – Core Types & Config
- [X] **1‑A Message Types**
  - [X] 1‑A‑1 Define `Role` enum and `Message` struct with Serde
  - [X] 1‑A‑2 Unit tests for (de)serialization
- [X] **1‑B Config Loader**
  - [X] 1‑B‑1 Design `Config` struct mirroring `config.toml`
  - [X] 1‑B‑2 Implement `Config::load()` reading `CONFIG_PATH` env (fallback default)
  - [X] 1‑B‑3 Unit tests with temp file
- [X] **1‑C Session Store**
  - [X] 1‑C‑1 Create in‑memory `SessionStore` (`HashMap<Uuid, Vec<Message>>`)
  - [X] 1‑C‑2 Add TTL cleanup (async task)
  - [X] 1‑C‑3 Unit tests (get/append/gc)

---

## Phase 2 – Server Skeleton
- [X] **2‑A Axum Setup**
  - [X] 2‑A‑1 Add `axum`, `tokio`, `tower` deps
  - [X] 2‑A‑2 `/health` route returning `"ok"`
  - [X] 2‑A‑3 Unit test `/health`
- [X] **2‑B SSE Utilities**
  - [X] 2‑B‑1 Implement helper to stream SSE events
  - [X] 2‑B‑2 Write simple SSE demo test
- [X] **2‑C Stub `/predict_stream`**
  - [X] 2‑C‑1 Define request/response models
  - [X] 2‑C‑2 Return static `"stub"` `assistant_output` stream
  - [X] 2‑C‑3 Integration test via reqwest

---

## Phase 3 – Embeddings
- [X] **3‑A Text Chunker**
  - [X] 3‑A‑1 Implement overlapping token chunk algorithm (~500/100)
  - [X] 3‑A‑2 Unit test chunk sizing & overlap
- [X] **3‑B Cohere Client**
  - [X] 3‑B‑1 Add Cohere API client (async)
  - [X] 3‑B‑2 Function `embed(texts) -> Vec<Vec<f32>>`
  - [X] 3‑B‑3 Handle API errors & retries
- [X] **3‑C Fallback Embeddings**
  - [X] 3‑C‑1 Stub fallback provider (returns zeros) for offline tests

---

## Phase 4 – Vector Store (pgvector)
- [X] **4‑A DB Migration**
  - [X] 4‑A‑1 Add `sqlx` + `postgres` driver
  - [X] 4‑A‑2 Create `documents` table with `embedding` vector(768)
- [X] **4‑B Insert Pipeline**
  - [X] 4‑B‑1 Function to persist chunk + embedding
  - [X] 4‑B‑2 CLI or script to ingest `./data/faq_docs/*.txt`
- [X] **4‑C Similarity Search**
  - [X] 4‑C‑1 `search_embeddings(query, k)` returning top‑k chunks
  - [X] 4‑C‑2 Unit tests with seeded data

---

## Phase 5 – LLM Wrapper (Bedrock Claude)
- [X] **5‑A Credentials & Client**
  - [X] 5‑A‑1 Load Bedrock creds from env / config
  - [X] 5‑A‑2 HTTP client setup
- [X] **5‑B Sonnet 4 Call**
  - [X] 5‑B‑1 Implement `call_claude(model, messages) -> Stream<String>`
  - [X] 5‑B‑2 Parse streaming chunks
- [X] **5‑C Fallback Logic**
  - [X] 5‑C‑1 Retry with Sonnet 3.7 on error/timeout
  - [X] 5‑C‑2 Unit tests using mock server

---

## Phase 6 – Tooling
- [X] **6‑A Tool Trait & Registry**
  - [X] 6‑A‑1 Define `Tool` trait (`name`, `execute`, `schema`)
  - [X] 6‑A‑2 Implement `ToolRegistry`
- [X] **6‑B Implement `file_summarizer`**
  - [X] 6‑B‑1 Read file, chunk, call LLM for summary
  - [X] 6‑B‑2 Return summary string
  - [X] 6‑B‑3 Unit test with fixture file
- [X] **6‑C Keyword Detector**
  - [X] 6‑C‑1 Simple heuristic to trigger `file_summarizer`
  - [X] 6‑C‑2 Unit tests for detector

---

## Phase 7 – Agent Loop & Streaming
- [X] **7‑A Integrate Retrieval + Tool + LLM**
  - [X] 7‑A‑1 Load session history
  - [X] 7‑A‑2 Detect and run tool (single‑step)
  - [X] 7‑A‑3 Retrieve top‑k chunks from pgvector
  - [X] 7‑A‑4 Compose prompt & call LLM
- [X] **7‑B Stream `tool_usage` Event**
  - [X] 7‑B‑1 Serialize tool metadata struct
  - [X] 7‑B‑2 Send SSE event before LLM call
- [ ] **7‑C Stream `assistant_output` Tokens**
  - [ ] 7‑C‑1 Proxy Claude stream to SSE
  - [ ] 7‑C‑2 Ensure graceful shutdown on disconnect

---

## Phase 8 – Polish, Docs & Tests
- [X] **8‑A Error Handling Hardening**
  - [X] 8‑A‑1 Define error enum with `thiserror`
  - [X] 8‑A‑2 Map errors to HTTP codes & SSE events
- [X] **8‑B Integration Test Suite**
  - [X] 8‑B‑1 Docker‑compose Postgres with pgvector
  - [X] 8‑B‑2 Mock Cohere & Bedrock
  - [X] 8‑B‑3 Test end‑to‑end chat happy‑path
- [X] **8‑C Documentation & Samples**
  - [X] 8‑C‑1 Write README with setup + run instructions
  - [X] 8‑C‑2 Provide sample `config.toml`
  - [ ] 8‑C‑3 Update CI to run integration tests

---

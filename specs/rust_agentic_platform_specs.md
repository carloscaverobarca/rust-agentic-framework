# 💬 Agentic Chatbot Backend — MVP Specification

**Version:** 1.0
**Author:** Carlos Cavero Barca
**Language:** Rust
**Framework:** Axum
**Purpose:** Company FAQ assistant with LLM-powered answers and tool use, exposed via a streaming API

---

## 🧭 1. Overview

Build a chatbot backend in Rust that:

* Answers FAQs from local raw text documents using retrieval-augmented generation (RAG)
* Integrates Bedrock Claude Sonnet 4 for synthesis (fallback: Sonnet 3.7)
* Uses Cohere embeddings with pgvector for semantic search
* Includes a single callable tool (`file_summarizer`)
* Exposes a single SSE-capable API endpoint: `POST /predict_stream`
* Supports in-memory session-based chat history and basic tool metadata streaming

---

## 🏗 2. Architecture

```
Client ↔ Axum Backend ↔ Bedrock API
                      ↘︎
                   Tool Executor (Rust)
                      ↘︎
                  Embedding + Retrieval (pgvector)
```

### Components

| Module           | Responsibility                                  |
| ---------------- | ----------------------------------------------- |
| Axum Web Server  | Handles `/predict_stream`, manages SSE          |
| Embedding Module | Chunk text, embed with Cohere                   |
| Vector Store     | Store and retrieve embeddings using pgvector    |
| LLM Client       | Call Bedrock Claude (v4 or fallback 3.7)        |
| Tool System      | File summarizer with schema-based Rust function |
| Session Store    | In-memory chat history per session              |

---

## 🔌 3. API Specification

### `POST /predict_stream`

**Headers**:

* `Content-Type: application/json`
* `Accept: text/event-stream`

**Request Body**:

```json
{
  "session_id": "string",
  "messages": [
    { "role": "user", "content": "How do I submit expenses?" }
  ]
}
```

**Streamed Response Events**:

* `tool_usage`

```json
{
  "tool": "file_summarizer",
  "args": { "file_path": "onboarding.txt" },
  "duration_ms": 123,
  "result": "Summarized content..."
}
```

* `assistant_output`

```json
{ "content": "Here’s a summary of the onboarding process..." }
```

---

## 📁 4. Data & Storage

### Input Documents

* Format: `.txt` files
* Location: Local directory (`./data/faq_docs`)
* Chunking: Overlapping text chunks (\~500 tokens with 100 overlap)
* Embeddings: Generated using Cohere
* Vector store: pgvector (PostgreSQL with `pgvector` extension)

### Embedding Record Schema (suggested)

```sql
CREATE TABLE documents (
  id UUID PRIMARY KEY,
  file_name TEXT,
  chunk_id INT,
  content TEXT,
  embedding VECTOR(768),
  created_at TIMESTAMP
);
```

---

## 🛠 5. Tool: `file_summarizer`

### Schema

```rust
#[derive(Deserialize)]
struct FileSummarizerInput {
    file_path: String,
}
```

### Function Signature

```rust
fn file_summarizer(input: FileSummarizerInput) -> Result<String, ToolError>
```

### Behavior

* Read file contents
* Chunk & send to Claude v4 for summarization
* Return plain-text summary

---

## 🧠 6. LLM Invocation

* Primary model: `Claude Sonnet v4` (Bedrock)
* Fallback: `Claude Sonnet v3.7`
* Use shared system prompt
* Messages sent in OpenAI-style format:

```json
{ "role": "user" | "assistant" | "tool", "content": "...", "name"?: "tool_name" }
```

### System Prompt (inject at beginning)

```
You are a helpful company assistant. Use the provided context to answer clearly and concisely.
```

---

## 🧩 7. Agent Logic (Simplified)

* Inspect `messages` for keywords like "summarize"
* If tool match:

    * Call `file_summarizer`
    * Insert result as a `tool` role message
    * Stream tool metadata
* Send full message list to Bedrock
* Stream `assistant_output` content as it arrives

---

## 🧠 8. Session Memory

* Stored in `HashMap<SessionId, Vec<Message>>`
* Stored in backend memory
* Optional TTL cleanup with async task

---

## ⚠ 9. Error Handling

| Component      | Error             | Response Strategy                      |
| -------------- | ----------------- | -------------------------------------- |
| Tool Call      | Invalid file path | Stream `tool_usage` with error details |
| Embedding      | API failure       | Return 500 with retryable hint         |
| LLM Call       | Bedrock timeout   | Retry with fallback model (3.7)        |
| Session Lookup | Missing session   | Start new session                      |
| SSE Failure    | Client disconnect | Graceful stream closure                |

---

## 🧪 10. Testing Plan

### Unit Tests

* [ ] Tool logic: `file_summarizer`
* [ ] Embedding + chunking
* [ ] LLM wrapper fallback
* [ ] SSE stream formatting

### Integration Tests

* [ ] Full chat flow with tool usage
* [ ] Vector search returns expected chunks
* [ ] LLM handles inserted context properly

### Manual Tests

* [ ] Trigger tool use via prompt
* [ ] Streamed tool metadata visible
* [ ] Claude fallback confirmed
* [ ] Rehydration of session from memory

---

## 🧰 11. Config File (`config.toml`)

```toml
[embedding]
provider = "cohere"

[llm]
primary = "claude-sonnet-v4"
fallback = "claude-sonnet-v3.7"

[pgvector]
url = "postgres://localhost:5432/chatbot"

[data]
document_dir = "./data/faq_docs"
```

---

## ✅ Ready to Start?

Let me know if you'd like a Rust scaffold with:

* Axum SSE handler
* pgvector wrapper
* Bedrock + Cohere clients
* Chat state + tool registration

Or I can walk you through implementation in stages.

# Agentic Chatbot Framework

An intelligent chatbot backend built in Rust that combines RAG (Retrieval-Augmented Generation) with tool use capabilities. The system integrates AWS Bedrock Claude Sonnet, Cohere embeddings, PostgreSQL with pgvector, and Server-Sent Events for streaming responses.

## Architecture Overview

This agentic chatbot provides:

- **RAG (Retrieval-Augmented Generation)** for answering questions from local document knowledge base
- **AWS Bedrock Claude Sonnet 4** for LLM synthesis with fallback to Sonnet 3.7
- **Cohere embeddings** with pgvector for semantic search
- **File summarizer tool** for document processing
- **SSE streaming API** via Axum framework for real-time responses

## Project Structure

```
agentic-framework/
├── Cargo.toml              # workspace root
├── crates/
│   ├── agentic-core/       # shared types, config, session management
│   ├── server/             # Axum HTTP & SSE server (main binary)
│   ├── embeddings/         # Cohere client with fallback, text chunker
│   ├── vector_store/       # pgvector integration
│   ├── llm/                # AWS Bedrock Claude wrapper with streaming
│   └── tooling/            # file_summarizer tool + tool registry
├── docker-compose.yml      # PostgreSQL development environment
├── sql/                    # Database initialization scripts
├── documents/              # Sample knowledge base files
├── specs/                  # Technical specifications
├── web_gui/                # React TypeScript frontend (optional)
└── CLAUDE.md               # Development guidelines (TDD)
```

## Quick Start

### Prerequisites

1. **Rust** (latest stable version)
2. **Docker & Docker Compose** (for PostgreSQL + pgvector)
3. **AWS CLI** configured with Bedrock access
4. **Cohere API key**

### 1. Clone and Build

```bash
git clone <repository-url>
cd agentic-framework
cargo build --release
```

### 2. Set Up PostgreSQL Database

Start the PostgreSQL database with pgvector extension:

```bash
docker-compose up -d
```

This creates a PostgreSQL database with:
- **Host**: localhost:5432 
- **Database**: chatbot
- **User**: postgres
- **Password**: postgres
- **pgvector extension** enabled
- **768-dimensional vector** support (Cohere embedding size)

### 3. Configure Environment

Create a `config.toml` file in the project root:

```toml
[embedding]
provider = "fallback"  # or "cohere" for production

[llm]
primary = "anthropic.claude-sonnet-4-20250514-v1:0"
fallback = "anthropic.claude-3-7-sonnet-20250219-v1:0"

[pgvector]
url = "postgresql://postgres:postgres@localhost:5432/chatbot"

[data]
document_dir = "./documents"
```

Set up your environment variables:

```bash
export COHERE_API_KEY="your-cohere-api-key"
export AWS_PROFILE="your-aws-profile-name"  # Optional, defaults to 'default'
export AWS_REGION="eu-central-1"
```

#### LLM model overrides via environment variables

The LLM model IDs from the `[llm]` section in `config.toml` can be overridden at runtime using environment variables. This is useful for switching models between environments (e.g., staging vs. prod) without changing files.

Set these variables before running the server:

```bash
export LLM_PRIMARY_MODEL="anthropic.claude-sonnet-4-20250514-v1:0"
export LLM_FALLBACK_MODEL="anthropic.claude-3-7-sonnet-20250219-v1:0"
```

Precedence:
- If set, `LLM_PRIMARY_MODEL` and `LLM_FALLBACK_MODEL` take precedence over `config.toml`.
- If not set, values from `config.toml` are used.

Implementation detail: overrides are applied centrally in configuration (`agentic-core`) via `LlmConfig::with_env_overrides()` and then passed to the LLM client during server startup.

#### Database URL override via environment variable

The `[pgvector].url` in `config.toml` can be overridden using the `PGVECTOR_URL` environment variable.

```bash
export PGVECTOR_URL="postgresql://username:password@host:5432/chatbot"
```

If `PGVECTOR_URL` is not set, the value from `config.toml` is used. The override is applied in `agentic-core` via `PgVectorConfig::with_env_overrides()` and used when initializing the vector store.

### 4. Initialize Document Directory

Create a documents directory and add your knowledge base files:

```bash
mkdir documents
echo "Company Policy: Remote work is allowed up to 3 days per week." > documents/hr_policy.txt
echo "Tech Stack: We use Rust for backend, TypeScript for frontend." > documents/tech_info.txt
```

### 5. Run the Server

```bash
cargo run --bin server
```

The server starts on `http://localhost:3000`

## Configuration Guide

### AWS Bedrock Setup

#### 1. Enable Claude Models

In the AWS Console:
1. Go to **AWS Bedrock** → **Model access**
2. Request access to:
   - `anthropic.claude-sonnet-4-20250514-v1:0` (Claude Sonnet v4)
   - `anthropic.claude-3-7-sonnet-20250219-v1:0` (Claude Sonnet v3.7)

#### 2. Configure AWS Credentials

**Option A: AWS CLI (Recommended)**
```bash
aws configure
```

**Option B: AWS Profile**
The application will automatically use your configured AWS profile. Specify which profile to use:

```bash
export AWS_PROFILE="your-profile-name"  # Optional, defaults to 'default'
export AWS_REGION="eu-central-1"
```

**Option C: IAM Role** (for EC2/ECS deployment)
Attach an IAM role with the `bedrock:InvokeModel` permission.

#### 3. Required IAM Permissions

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "bedrock:InvokeModel",
                "bedrock:InvokeModelWithResponseStream"
            ],
            "Resource": [
                "arn:aws:bedrock:*::foundation-model/anthropic.claude-sonnet-4-20250514-v1:0",
                "arn:aws:bedrock:*::foundation-model/anthropic.claude-3-7-sonnet-20250219-v1:0"
            ]
        }
    ]
}
```

### Cohere Setup

#### 1. Get API Key

1. Sign up at [cohere.com](https://cohere.com)
2. Navigate to **API Keys** in the dashboard
3. Create a new API key

#### 2. Set Environment Variable

```bash
export COHERE_API_KEY="your-cohere-api-key"
```

#### 3. Configure in config.toml

```toml
[embedding]
provider = "cohere"
```

**Alternative: Use Fallback Provider**

For testing without Cohere API:

```toml
[embedding]
provider = "fallback"
```

### Database Configuration

#### PostgreSQL with pgvector (Production)

1. **Install PostgreSQL** with pgvector extension
2. **Create Database**:

```sql
CREATE DATABASE chatbot;
CREATE EXTENSION vector;

CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name TEXT NOT NULL,
    chunk_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding VECTOR(768),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX documents_embedding_idx 
ON documents USING ivfflat (embedding vector_cosine_ops) 
WITH (lists = 100);
```

3. **Configure Connection**:

```toml
[pgvector]
url = "postgresql://username:password@localhost:5432/chatbot"
```

#### Docker Setup (Development)

Use the included docker-compose for local development:

```bash
docker-compose up -d
```

This automatically:
- Sets up PostgreSQL with pgvector extension
- Creates the database schema
- Configures proper vector indexes
- Ready for development and testing

## API Usage

### Health Check

```bash
curl http://localhost:3000/health
```

### Chat with Streaming Response

The main endpoint uses Server-Sent Events (SSE) for streaming responses:

```bash
curl -N -H "Content-Type: application/json" \
     -H "Accept: text/event-stream" \
     -d '{
       "session_id": "550e8400-e29b-41d4-a716-446655440000",
       "messages": [
         {
           "role": "User", 
           "content": "What is our remote work policy?",
           "name": null
         }
       ]
     }' \
     http://localhost:3000/predict_stream
```

### Example with Tool Usage

Request that triggers file summarization:

```bash
curl -N -H "Content-Type: application/json" \
     -H "Accept: text/event-stream" \
     -d '{
       "session_id": "550e8400-e29b-41d4-a716-446655440001",
       "messages": [
         {
           "role": "User",
           "content": "Can you summarize the hr_policy.txt file?",
           "name": null
         }
       ]
     }' \
     http://localhost:3000/predict_stream
```

### SSE Response Format

The server returns Server-Sent Events in this format:

```
event: tool_usage
data: {"tool": "file_summarizer", "args": {"file_path": "./documents/hr_policy.txt"}, "result": "File summary...", "duration_ms": 150}

event: assistant_output  
data: {"content": "Based on the HR policy document, our remote work policy allows..."}
```

### Complex Query Example

Test both document retrieval AND tool usage:

```bash
curl -N -H "Content-Type: application/json" \
     -H "Accept: text/event-stream" \
     -d '{
       "session_id": "550e8400-e29b-41d4-a716-446655440002",
       "messages": [
         {
           "role": "User",
           "content": "Summarize hr_policy.txt and tell me about our vacation policy",
           "name": null
         }
       ]
     }' \
     http://localhost:3000/predict_stream
```

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires PostgreSQL)
docker-compose up -d
cargo test --package server --test integration_tests
```

### Building for Production

```bash
cargo build --release
```

The binary will be available at `target/release/server`.

### Adding Documents

Add documents to your knowledge base:

```bash
# Add documents to the configured document_dir
echo "New company policy..." > documents/new_policy.txt

# The server will automatically process them when referenced in queries
```

### Supported File Types

The file summarizer tool supports:
- `.txt` - Plain text files
- `.md` - Markdown files  
- `.rs` - Rust source files (with function/struct/impl counting)
- `.py` - Python files (with function/class/import counting)
- `.js`, `.ts` - JavaScript/TypeScript files
- `.json` - JSON configuration files
- `.yaml`, `.yml` - YAML files
- `.toml` - TOML configuration files
- `.cfg`, `.conf` - Configuration files

## Error Handling

The system includes comprehensive error handling:

### Error Types

- **EmbeddingError**: Issues with Cohere API or embedding generation
- **LlmError**: AWS Bedrock connectivity or model issues
- **DatabaseError**: PostgreSQL connection or query failures  
- **ToolError**: File access or tool execution problems
- **ConfigError**: Configuration file or environment issues

### Error Responses

Errors are returned as SSE events:

```
event: error_event
data: {"error_type": "LlmError", "message": "AWS Bedrock timeout", "retryable": true, "http_status": 503}
```

## Troubleshooting

### Common Issues

#### "Failed to connect to PostgreSQL"
- Ensure PostgreSQL is running: `docker-compose ps`
- Check connection string in `config.toml`
- Verify pgvector extension: `psql -c "SELECT * FROM pg_extension WHERE extname='vector';"`
- For testing with fallback: Use SQLite URL in config for in-memory testing

#### "Cohere API error"  
- Verify API key: `echo $COHERE_API_KEY`
- Check API quota at cohere.com dashboard
- Use fallback provider for testing: Set `provider = "fallback"` in config

#### "AWS Bedrock 403 Forbidden"
- Verify AWS credentials: `aws sts get-caller-identity`
- Check Bedrock model access in AWS Console
- Ensure proper IAM permissions for `bedrock:InvokeModel`

#### "Tool execution failed"
- Check document directory exists: `ls -la documents/`
- Verify file permissions
- Check file paths are relative to `document_dir`

### Logs and Debugging

Enable debug logging:

```bash
RUST_LOG=debug cargo run --bin server
```

## Deployment

### Docker Deployment

Create a `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY config.toml /app/config.toml
WORKDIR /app
CMD ["server"]
```

### Environment Variables

For production deployment:

```bash
export RUST_LOG=info
export COHERE_API_KEY="production-key"
export AWS_PROFILE="production-profile"  # Optional, defaults to 'default'
export AWS_REGION="eu-central-1" 
export DATABASE_URL="postgresql://user:pass@prod-db:5432/chatbot"
```

## Performance Considerations

### Vector Search Optimization

- The pgvector index uses **IVFFlat** with 100 lists
- Optimal for up to ~1M documents
- For larger datasets, consider **HNSW** indexing

### Embedding Batch Size

- Cohere processes up to 96 texts per request
- Text chunking uses 500-character chunks with 100-character overlap
- Adjust `ChunkConfig` for your document types

### Database Connection Pooling

- Uses sqlx connection pool (default: 10 connections)
- Adjust pool size based on concurrent load
- Monitor connection usage in production

## Web GUI

The project includes an optional React TypeScript frontend in the `web_gui/` directory:

- **React 18** with TypeScript
- **Vite 4** for development and building  
- **SSE client** for real-time streaming responses
- **Modern UI** with responsive design and chat interface

### Running the Web GUI

```bash
cd web_gui

# Install dependencies
npm install

# Start development server (requires backend running on port 3000)
npm run dev
```

The web interface will be available at `http://localhost:5173` (or next available port like 5174) with proxy forwarding to the Rust backend.

### Web GUI Features

- Real-time chat interface with streaming responses
- Session management with UUID tracking
- Tool usage indicators (file summarization)
- Error handling and retry logic
- Responsive design for mobile and desktop

## Key Dependencies

The project uses:
- **axum** (0.7) - Web framework for HTTP and SSE
- **tokio** (1.0) - Async runtime with full feature set
- **sqlx** (0.8) - Database toolkit with PostgreSQL support
- **pgvector** (0.4) - PostgreSQL vector extension bindings
- **serde** (1.0) - JSON serialization/deserialization
- **uuid** (1.0) - Session and document ID generation
- **aws-sdk-bedrockruntime** (1.15) - AWS Bedrock Claude models
- **reqwest** (0.11) - HTTP client for Cohere API
- **async-stream** (0.3) - Stream utilities for SSE
- **chrono** (0.4) - Date/time handling
- **anyhow**/**thiserror** - Error handling
- **log**/**env_logger** - Logging framework

## Contributing

1. **Follow TDD methodology** as outlined in `CLAUDE.md`
2. **Use functional style** - prefer Option/Result combinators over pattern matching
3. **Run tests**: `cargo test` (all unit tests pass)
4. **Integration tests**: `cargo test --package server --test integration_tests`
5. **Format code**: `cargo fmt`
6. **Lint code**: `cargo clippy`
7. **Follow Kent Beck's principles**: Red → Green → Refactor
8. **Separate structural from behavioral changes** in commits

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

## Support

For issues and questions:
- Create GitHub issues for bugs
- Check troubleshooting section for common problems
- Review integration tests for usage examples
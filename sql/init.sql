-- Initialize the test database with pgvector extension and tables

-- Enable the pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create the documents table for embeddings
-- NOTE: This schema is for reference only. The application now dynamically creates
-- tables with the correct embedding dimensions (1024 for all providers: Cohere v3/Bedrock/Fallback)
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name TEXT NOT NULL,
    chunk_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding VECTOR(1024), -- All embedding providers now use 1024 dimensions
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_documents_file_name ON documents(file_name);
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at);

-- Create optimized vector similarity search index
CREATE INDEX IF NOT EXISTS documents_embedding_idx ON documents
USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 50);  -- Optimized for better recall with Titan embeddings

-- Configure vector search parameters
ALTER DATABASE test_chatbot SET ivfflat.probes = 10;

-- Grant necessary permissions to test user
GRANT ALL PRIVILEGES ON DATABASE test_chatbot TO test_user;
GRANT ALL PRIVILEGES ON TABLE documents TO test_user;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO test_user;
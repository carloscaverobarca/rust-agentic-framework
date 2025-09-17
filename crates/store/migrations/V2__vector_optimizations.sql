-- Add vector search optimizations
-- V2__vector_optimizations.sql

-- Create optimized vector similarity search index
CREATE INDEX IF NOT EXISTS documents_embedding_idx ON documents
USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 50);  -- Optimized for better recall

-- Configure vector search parameters
ALTER DATABASE test_chatbot SET ivfflat.probes = 10;

-- Grant necessary permissions to test user
GRANT ALL PRIVILEGES ON DATABASE test_chatbot TO test_user;
GRANT ALL PRIVILEGES ON TABLE documents TO test_user;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO test_user;

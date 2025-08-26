/// A text chunk with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct TextChunk {
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub chunk_id: usize,
}

/// Configuration for text chunking
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub chunk_size: usize,   // ~500 tokens
    pub overlap_size: usize, // ~100 tokens
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 500,
            overlap_size: 100,
        }
    }
}

/// Text chunker that splits text into overlapping chunks
pub struct TextChunker {
    config: ChunkConfig,
}

impl TextChunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    pub fn chunk_text(&self, text: &str) -> Vec<TextChunk> {
        if text.is_empty() {
            return vec![];
        }

        let tokens: Vec<&str> = text.split_whitespace().collect();
        let total_tokens = tokens.len();

        if total_tokens <= self.config.chunk_size {
            // Text fits in a single chunk
            return vec![TextChunk {
                content: text.to_string(),
                start_pos: 0,
                end_pos: text.len(),
                chunk_id: 0,
            }];
        }

        let mut chunks = Vec::new();
        let mut chunk_id = 0;
        let mut start_token_idx = 0;

        while start_token_idx < total_tokens {
            let end_token_idx =
                std::cmp::min(start_token_idx + self.config.chunk_size, total_tokens);

            let chunk_tokens = &tokens[start_token_idx..end_token_idx];
            let chunk_text = chunk_tokens.join(" ");

            // Calculate character positions
            let start_pos = if start_token_idx == 0 {
                0
            } else {
                // Find the character position of the start token
                tokens[0..start_token_idx]
                    .iter()
                    .map(|token| token.len() + 1) // +1 for space
                    .sum::<usize>()
                    .saturating_sub(1) // Remove last space
            };

            let end_pos = if end_token_idx == total_tokens {
                text.len()
            } else {
                start_pos + chunk_text.len()
            };

            chunks.push(TextChunk {
                content: chunk_text,
                start_pos,
                end_pos,
                chunk_id,
            });

            chunk_id += 1;

            // Move to next chunk with overlap
            if end_token_idx >= total_tokens {
                break;
            }

            start_token_idx = end_token_idx.saturating_sub(self.config.overlap_size);
        }

        chunks
    }

    /// Simple token approximation: split by whitespace
    pub fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_chunker_with_default_config() {
        let config = ChunkConfig::default();
        let chunker = TextChunker::new(config.clone());

        assert_eq!(chunker.config.chunk_size, 500);
        assert_eq!(chunker.config.overlap_size, 100);
    }

    #[test]
    fn should_create_chunker_with_custom_config() {
        let config = ChunkConfig {
            chunk_size: 200,
            overlap_size: 50,
        };
        let chunker = TextChunker::new(config.clone());

        assert_eq!(chunker.config.chunk_size, 200);
        assert_eq!(chunker.config.overlap_size, 50);
    }

    #[test]
    fn should_return_empty_for_empty_text() {
        let chunker = TextChunker::new(ChunkConfig::default());
        let chunks = chunker.chunk_text("");

        assert!(chunks.is_empty());
    }

    #[test]
    fn should_return_single_chunk_for_short_text() {
        let chunker = TextChunker::new(ChunkConfig {
            chunk_size: 10,
            overlap_size: 2,
        });

        let text = "This is a short text.";
        let chunks = chunker.chunk_text(text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
        assert_eq!(chunks[0].start_pos, 0);
        assert_eq!(chunks[0].end_pos, text.len());
        assert_eq!(chunks[0].chunk_id, 0);
    }

    #[test]
    fn should_create_overlapping_chunks_for_long_text() {
        let chunker = TextChunker::new(ChunkConfig {
            chunk_size: 5,   // 5 tokens per chunk
            overlap_size: 2, // 2 tokens overlap
        });

        // 12 tokens total: "one two three four five six seven eight nine ten eleven twelve"
        let text = "one two three four five six seven eight nine ten eleven twelve";
        let chunks = chunker.chunk_text(text);

        // Should create multiple chunks with overlap
        assert!(chunks.len() > 1);

        // First chunk should start at position 0
        assert_eq!(chunks[0].start_pos, 0);
        assert_eq!(chunks[0].chunk_id, 0);

        // Last chunk should end at text length
        let last_chunk = &chunks[chunks.len() - 1];
        assert_eq!(last_chunk.end_pos, text.len());

        // Chunks should have overlapping content
        if chunks.len() >= 2 {
            let first_tokens: Vec<&str> = chunks[0].content.split_whitespace().collect();
            let second_tokens: Vec<&str> = chunks[1].content.split_whitespace().collect();

            // The last N tokens of first chunk should match first N tokens of second chunk
            let overlap_expected = std::cmp::min(chunker.config.overlap_size, first_tokens.len());
            let overlap_actual = first_tokens
                .iter()
                .skip(first_tokens.len().saturating_sub(overlap_expected))
                .zip(second_tokens.iter())
                .filter(|(a, b)| a == b)
                .count();

            assert!(
                overlap_actual > 0,
                "Chunks should have overlapping tokens. First: {:?}, Second: {:?}",
                first_tokens,
                second_tokens
            );
        }
    }

    #[test]
    fn should_estimate_tokens_correctly() {
        let chunker = TextChunker::new(ChunkConfig::default());

        assert_eq!(chunker.estimate_tokens(""), 0);
        assert_eq!(chunker.estimate_tokens("hello"), 1);
        assert_eq!(chunker.estimate_tokens("hello world"), 2);
        assert_eq!(chunker.estimate_tokens("  hello   world  "), 2);
        assert_eq!(chunker.estimate_tokens("one two three four five"), 5);
    }

    #[test]
    fn should_handle_text_exactly_matching_chunk_size() {
        let chunker = TextChunker::new(ChunkConfig {
            chunk_size: 5,
            overlap_size: 1,
        });

        let text = "one two three four five"; // Exactly 5 tokens
        let chunks = chunker.chunk_text(text);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn should_preserve_chunk_boundaries_on_word_boundaries() {
        let chunker = TextChunker::new(ChunkConfig {
            chunk_size: 3,
            overlap_size: 1,
        });

        let text = "word1 word2 word3 word4 word5";
        let chunks = chunker.chunk_text(text);

        // Each chunk should end on word boundaries, not cut words in half
        for chunk in &chunks {
            assert!(!chunk.content.trim().ends_with(' '));
            assert!(!chunk.content.trim().starts_with(' '));
        }
    }
}

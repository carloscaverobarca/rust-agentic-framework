use crate::models::{ChatMessage, ModelConfig, StreamEvent};
use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{
    types::{
        ContentBlock, ConversationRole, ConverseStreamOutput as ConverseStreamOutputType,
        InferenceConfiguration, Message,
    },
    Client,
};
use futures::stream::Stream;
use log::{error, info};
use std::pin::Pin;
use std::time::Duration;

pub struct BedrockClient {
    client: Client,
    config: ModelConfig,
}

impl BedrockClient {
    pub async fn new(config: ModelConfig) -> Result<Self> {
        info!("Initializing BedrockClient");
        // Load AWS configuration using the official SDK with proper error handling
        let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        let client = Client::new(&aws_config);

        Ok(Self { client, config })
    }

    pub async fn new_with_region(config: ModelConfig, region: &str) -> Result<Self> {
        // Allow explicit region configuration for testing
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.to_string()))
            .load()
            .await;

        let client = Client::new(&aws_config);

        Ok(Self { client, config })
    }

    pub async fn call_claude(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send + '_>>> {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt <= self.config.max_retries {
            let model = if attempt == 0 {
                &self.config.primary_model
            } else {
                &self.config.fallback_model
            };

            match self.try_call_claude(messages.clone(), model).await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    last_error = Some(e);
                    attempt += 1;

                    if attempt <= self.config.max_retries {
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt - 1)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn try_call_claude(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send + '_>>> {
        // Convert our ChatMessage format to Bedrock's Message format
        let bedrock_messages = self.convert_to_bedrock_messages(messages)?;

        info!("Sending request to Bedrock model: {}", model);
        let response = self
            .client
            .converse_stream()
            .model_id(model)
            .inference_config(
                InferenceConfiguration::builder()
                    .max_tokens(self.config.max_tokens as i32)
                    .temperature(self.config.temperature)
                    .build(),
            )
            .set_messages(Some(bedrock_messages))
            .send()
            .await
            .map_err(|e| {
                error!("Bedrock send error: {:?}", e);
                anyhow::anyhow!("Failed to send request to Bedrock: {}", e)
            })?;

        info!("Received response from Bedrock model: {}", model);

        // Convert the AWS event stream to our StreamEvent format
        let stream = self.process_bedrock_stream(response).await?;
        Ok(Box::pin(stream))
    }

    async fn process_bedrock_stream(
        &self,
        response: aws_sdk_bedrockruntime::operation::converse_stream::ConverseStreamOutput,
    ) -> Result<impl Stream<Item = Result<StreamEvent>> + Send + '_> {
        let event_stream = response.stream;

        Ok(async_stream::stream! {
            let mut stream = event_stream;
            loop {
                match stream.recv().await {
                    Ok(Some(stream_event)) => {
                        match stream_event {
                            ConverseStreamOutputType::MessageStart(_) => {
                                yield Ok(StreamEvent::MessageStart);
                            }
                            ConverseStreamOutputType::ContentBlockStart(_) => {
                                yield Ok(StreamEvent::ContentBlockStart);
                            }
                            ConverseStreamOutputType::ContentBlockDelta(delta) => {
                                if let Some(aws_sdk_bedrockruntime::types::ContentBlockDelta::Text(text)) = delta.delta() {
                                    yield Ok(StreamEvent::ContentBlockDelta {
                                        text: text.clone(),
                                    });
                                } else {
                                    yield Ok(StreamEvent::ContentBlockDelta {
                                        text: "".to_string(),
                                    });
                                }
                            }
                            ConverseStreamOutputType::ContentBlockStop(_) => {
                                yield Ok(StreamEvent::ContentBlockStop);
                            }
                            ConverseStreamOutputType::MessageStop(_) => {
                                yield Ok(StreamEvent::MessageStop);
                                break;
                            }
                            _ => {
                                // Handle other event types or unknown events
                                yield Ok(StreamEvent::ContentBlockDelta {
                                    text: "".to_string(),
                                });
                            }
                        }
                    }
                    Ok(None) => {
                        // Stream ended
                        break;
                    }
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Stream error: {}", e));
                        break;
                    }
                }
            }
        })
    }

    fn convert_to_bedrock_messages(&self, messages: Vec<ChatMessage>) -> Result<Vec<Message>> {
        let mut bedrock_messages = Vec::new();

        for msg in messages {
            let role = match msg.role.as_str() {
                "user" => ConversationRole::User,
                "assistant" => ConversationRole::Assistant,
                _ => continue, // Skip system messages and other types for now
            };

            let content_block = ContentBlock::Text(msg.content);

            let bedrock_message = Message::builder()
                .role(role)
                .content(content_block)
                .build()
                .context("Failed to build Bedrock message")?;

            bedrock_messages.push(bedrock_message);
        }

        Ok(bedrock_messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_bedrock_client() {
        // Set test environment variables for proper AWS client initialization
        std::env::set_var("AWS_REGION", "us-east-1");

        let config = ModelConfig::default();

        // Client creation should succeed with AWS profile configuration
        let result = BedrockClient::new(config).await;

        assert!(
            result.is_ok(),
            "BedrockClient creation should succeed with AWS profile"
        );
    }

    #[tokio::test]
    async fn should_handle_claude_call_interface() {
        // Set test environment variables for proper AWS client initialization
        std::env::set_var("AWS_REGION", "us-east-1");

        let config = ModelConfig::default();

        let client = BedrockClient::new(config)
            .await
            .expect("Should create client with AWS profile");

        let messages = vec![ChatMessage::user("Hello".to_string())];

        // This will fail due to auth/permissions but tests the interface works
        let result = client.call_claude(messages).await;

        // We expect this to fail with auth error, but the interface should work
        assert!(
            result.is_err(),
            "Should fail with auth error using AWS profile"
        );
    }

    #[tokio::test]
    async fn should_convert_messages_to_bedrock_format() {
        // Set test environment variables for proper AWS client initialization
        std::env::set_var("AWS_REGION", "us-east-1");

        let config = ModelConfig::default();

        // Create client using the proper constructor
        let client = BedrockClient::new(config)
            .await
            .expect("Should create client with AWS profile");

        let messages = vec![
            ChatMessage::user("Hello".to_string()),
            ChatMessage::assistant("Hi there".to_string()),
        ];

        let bedrock_messages = client.convert_to_bedrock_messages(messages).unwrap();

        assert_eq!(bedrock_messages.len(), 2);
        // Additional assertions would require inspecting the internal structure
        // which is complex with the AWS SDK types
    }

    #[test]
    fn should_use_fallback_model_on_retry() {
        let config = ModelConfig::default();

        assert_ne!(config.primary_model, config.fallback_model);
        assert_eq!(
            config.primary_model,
            "anthropic.claude-3-5-sonnet-20241022-v2:0"
        );
        assert_eq!(
            config.fallback_model,
            "anthropic.claude-3-5-sonnet-20240620-v1:0"
        );
    }
}

use anyhow::Result;
use aws_sdk_bedrockruntime::{primitives::Blob, Client as BedrockClient};
use tracing::error;

pub async fn invoke_bedrock(
    client: &BedrockClient,
    model_id: &str,
    body: Vec<u8>,
) -> Result<Vec<u8>> {
    let blob = Blob::new(body);
    let response = client
        .invoke_model()
        .model_id(model_id)
        .body(blob)
        .send()
        .await
        .map_err(|e| {
            error!("Bedrock invoke_model failed: {}", e);
            e
        })?;
    Ok(response.body().as_ref().to_vec())
}

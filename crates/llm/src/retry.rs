use anyhow::Result;
use std::future::Future;
use std::time::Duration;
use tracing::{error, warn};

pub async fn retry_with_backoff<T, F, Fut>(max_retries: u32, mut f: F) -> Result<T>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match f(attempt).await {
            Ok(value) => return Ok(value),
            Err(e) => {
                warn!("Attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);

                if attempt < max_retries {
                    tokio::time::sleep(Duration::from_millis(1000 * 2_u64.pow(attempt))).await;
                }
            }
        }
    }

    let err = last_error.unwrap();
    error!("All retry attempts failed. Final error: {}", err);
    Err(err)
}

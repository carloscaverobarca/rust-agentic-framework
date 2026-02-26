use anyhow::Result;
use std::future::Future;
use std::time::Duration;

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
                last_error = Some(e);

                if attempt < max_retries {
                    tokio::time::sleep(Duration::from_millis(1000 * 2_u64.pow(attempt))).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

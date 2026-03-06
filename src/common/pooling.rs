use crate::error;
use tokio::time::{Duration, sleep};

pub(crate) async fn pool<T, FAction, FutA, FCond>(
    action: FAction,
    condition: FCond,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<T, &'static str>
where
    FAction: Fn() -> FutA,
    FutA: Future<Output = Result<T, error::ErrorResponse>>,
    FCond: Fn(&T) -> bool,
{
    for _ in 1..=max_attempts {
        match action().await {
            Ok(result) => {
                if condition(&result) {
                    return Ok(result);
                }
            }
            Err(_) => {
                return Err("get_status_error");
            }
        }

        sleep(Duration::from_millis(sleep_time)).await;
    }

    Err("Maximum number of attempts exceeded")
}

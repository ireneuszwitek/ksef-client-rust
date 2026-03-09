use crate::error;
use tokio::time::{Duration, sleep};

pub(crate) async fn pool<T, FAction, FutA, FCond>(
    action: FAction,
    condition: FCond,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<T, error::ErrorResponse>
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
            Err(e) => {
                return Err(e);
            }
        }

        sleep(Duration::from_millis(sleep_time)).await;
    }

    Err(error::ErrorResponse {
                    code: "max_attempts_exceeded".into(),
                    message: "Maximum number of attempts exceeded".into(),
    })
}

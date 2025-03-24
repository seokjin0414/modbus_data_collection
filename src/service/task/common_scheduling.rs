use std::future::Future;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use tokio::time::{interval_at, Duration, Instant};
use tracing::info;

use crate::service::server::get_state::ServerState;

pub const SECONDS_1MINUTE: u64 = 60;

pub async fn schedule_task<F, Fut>(
    state: Arc<ServerState>,
    task: F,
    task_descriptor: String,
    cycle_seconds: u64,
    delay_seconds: u64,
) -> Result<()>
where
    F: Fn(Arc<ServerState>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let initial_delay = next_run_time_delay(cycle_seconds as i64)
        .map_err(|e| anyhow!("{:?}", e))?;

    let total_initial_delay = initial_delay + Duration::from_secs(delay_seconds);

    let now = Utc::now();
    let first_run_time = now + chrono::Duration::from_std(total_initial_delay)
        .map_err(|e| anyhow!("Conversion failed: {:?}", e))?;

    info!("{:?}", schedule_message(&task_descriptor, now, first_run_time));

    let start_time = Instant::now() + total_initial_delay;
    let mut interval_timer = interval_at(start_time, Duration::from_secs(cycle_seconds));

    loop {
        interval_timer.tick().await;
        task(Arc::clone(&state)).await;
    }
}

pub fn schedule_message(task_descriptor: &str, now: DateTime<Utc>, next: DateTime<Utc>) -> String {
    let diff_sec = (next - now).num_seconds();
    format!(
        "Task '{}' will run in {} seconds (at {})",
        task_descriptor,
        diff_sec,
        next.to_rfc3339()
    )
}

pub fn next_run_time_delay(cycle_seconds: i64) -> Result<Duration> {
    let now = Utc::now();
    let next_timestamp = (now.timestamp() / cycle_seconds + 1) * cycle_seconds;

    let next_time = match Utc.timestamp_opt(next_timestamp, 0) {
        LocalResult::Single(ts) => ts,
        LocalResult::Ambiguous(ts_1, _) => ts_1,
        LocalResult::None => {
            return Err(anyhow!("Could not determine the next run time mark due to a time gap."));
        }
    };

    let delay = next_time - now;
    delay.to_std()
        .map_err(|e| anyhow!("Failed to convert delay to std duration: {:?}", e))
}
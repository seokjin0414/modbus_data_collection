use anyhow::Result;
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use tracing::{error, info};

use crate::service::{
    collect::gems_3500_modbus::collection_gems_3500_modbus,
    server::{get_state::ServerState, udp_listener::run_udp_listener},
    task::common_scheduling::{SECONDS_5MINUTE, schedule_task},
};

pub async fn task_init(state: Arc<ServerState>) -> Result<()> {
    info!("Task scheduler running...");

    {
        let coroutine_state = Arc::clone(&state);
        tokio::spawn(async move {
            schedule_task(
                Arc::clone(&coroutine_state),
                move |st| async move {
                    match collection_gems_3500_modbus(&st).await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Could not collection_gems_3500_modbus: {:?}", e);
                        }
                    }
                },
                String::from("collect modbus data from client server"),
                SECONDS_5MINUTE,
                0,
            )
            .await
        });
    }

    {
        let coroutine_state = Arc::clone(&state);
        tokio::spawn(async move {
            schedule_task(
                Arc::clone(&coroutine_state.clone()),
                move |st| async move {
                    let res = timeout(Duration::from_secs(30), run_udp_listener(st)).await;
                    match res {
                        Ok(Ok(())) => info!("UDP listener done"),
                        Ok(Err(e)) => error!("listener error: {:?}", e),
                        Err(_) => info!("UDP listener timed out after 30s"),
                    }
                },
                String::from("collect iaq data from client server"),
                SECONDS_5MINUTE,
                0,
            )
            .await
        });
    }

    Ok(())
}

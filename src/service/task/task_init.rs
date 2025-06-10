use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::service::{
    collect::gems_3500_modbus::collection_gems_3500_modbus,
    server::get_state::ServerState,
    task::common_scheduling::{SECONDS_1MINUTE, schedule_task},
};

pub async fn task_init(state: Arc<ServerState>) -> Result<()> {
    info!("Task scheduler running...");
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
            SECONDS_1MINUTE,
            0,
        )
        .await
    });

    Ok(())
}

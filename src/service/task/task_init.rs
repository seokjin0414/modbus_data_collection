use anyhow::Result;
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use tracing::{error, info};

use crate::service::{
    collect::{
        gas::handle_gas_data, gems_3500_modbus::collection_gems_3500_modbus, heat::handle_heat_data,
    },
    server::{get_state::ServerState, udp_listener::run_udp_listener},
    task::common_scheduling::{SECONDS_1MINUTE, SECONDS_5MINUTE, schedule_task},
};

pub async fn task_init(state: Arc<ServerState>) -> Result<()> {
    info!("Task scheduler running...");

    // {
    //     let coroutine_state = Arc::clone(&state);
    //     tokio::spawn(async move {
    //         schedule_task(
    //             Arc::clone(&coroutine_state),
    //             move |st| async move {
    //                 match collection_gems_3500_modbus(&st).await {
    //                     Ok(_) => (),
    //                     Err(e) => {
    //                         error!("Could not collection_gems_3500_modbus: {:?}", e);
    //                     }
    //                 }
    //             },
    //             String::from("collect modbus data from client server"),
    //             SECONDS_5MINUTE,
    //             0,
    //         )
    //         .await
    //     });
    // }

    // {
    //     let coroutine_state = Arc::clone(&state);
    //     tokio::spawn(async move {
    //         schedule_task(
    //             Arc::clone(&coroutine_state),
    //             move |st| async move {
    //                 match run_udp_listener(st).await {
    //                     Ok(_) => (),
    //                     Err(e) => {
    //                         error!("Could not collect iaq data: {:?}", e);
    //                     }
    //                 }
    //             },
    //             String::from("collect iaq data from client server"),
    //             SECONDS_5MINUTE,
    //             0,
    //         )
    //         .await
    //     });
    // }

    {
        let coroutine_state = Arc::clone(&state);
        tokio::spawn(async move {
            schedule_task(
                Arc::clone(&coroutine_state),
                move |st| async move {
                    match handle_heat_data(st).await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Could not collect heat data: {:?}", e);
                        }
                    }
                },
                String::from("collect heat modbus data from client server"),
                SECONDS_1MINUTE,
                0,
            )
            .await
        });
    }

    {
        let coroutine_state = Arc::clone(&state);
        tokio::spawn(async move {
            schedule_task(
                Arc::clone(&coroutine_state),
                move |st| async move {
                    match handle_gas_data(st).await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Could not collect gas data: {:?}", e);
                        }
                    }
                },
                String::from("collect gas modbus data from client server"),
                SECONDS_1MINUTE,
                0,
            )
            .await
        });
    }

    Ok(())
}

use anyhow::{anyhow, Result};
use tracing::info;
use crate::service::server::server_init::server_initializer;

mod model {
    pub mod gems_3005 {
        pub mod data_models;
        pub mod gems_3500_memory_map_models;
    }

    pub mod modbus {
        pub mod modbus_register_models;
    }
}

mod service {
    pub mod collect {
        pub mod gems_3500_modbus;
    }

    pub mod read {
        pub mod read_from_addr;
        pub mod read_from_register;
    }

    pub mod server {
        pub mod get_state;
        pub mod server_init;
    }

    pub mod task {
        pub mod common_scheduling;
        pub mod task_init;
    }

    pub mod interpret_modbus_register;
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let result = server_initializer()
        .await
        .map_err(|e| anyhow!("{:?}", e))?;

    info!("Server successfully terminated: {}", result);
    Ok(())
}
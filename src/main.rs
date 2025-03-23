use anyhow::anyhow;
use crate::service::collect::gems_3500_modbus::collection_gems_3500_modbus;

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

    pub mod interpret_modbus_register;
}

#[tokio::main]
async fn main() {
    let _ = collection_gems_3500_modbus()
        .await
        .map_err(|e| {
            println!("fail to collection_gems_3500_modbus: {:?}", e);
            anyhow!(e)
        });
}
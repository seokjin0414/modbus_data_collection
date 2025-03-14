use anyhow::anyhow;
use crate::service::collect::modbus_collect::modbus_collect;

mod model {
    pub mod gems_3005 {
        pub mod gems_3500_memory_map_models;
    }

    pub mod modbus {
        pub mod modbus_register_models;
    }
}

mod service {
    pub mod collect {
        pub mod modbus_collect;
    }

    pub mod read {
        pub mod read_from_addr;
        pub mod read_from_register;
    }

    pub mod interpret_modbus_register;
}

#[tokio::main]
async fn main() {
    let _ = modbus_collect()
        .await
        .map_err(|e| {
            println!("fail to bems_modbus_collect: {:?}", e);
            anyhow!(e)
        });
}
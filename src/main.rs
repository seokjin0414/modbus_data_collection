use anyhow::anyhow;

mod model {
    pub mod gems_3500_memory_map_schema_models;
    pub mod modbus_register_models;
}

mod service {
    pub mod interpret_modbus_register;
    pub mod read_from_addr;
    pub mod read_from_register;
}

#[tokio::main]
async fn main() {



}
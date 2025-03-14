use tokio_modbus::client::{Context, Reader};
use anyhow::{anyhow, Result};

use crate::model::modbus_register_models::ModbusRegisterType;
use crate::service::interpret_modbus_register::interpret_modbus_register_return_type;

// Note that modbus words are big-endian 16-bit values.
pub async fn read_from_register(
    ctx: &mut Context,
    reg_address: u16,
    value_type: ModbusRegisterType,
    divide_by: i16,
) -> Result<Option<f64>> {
    let data = ctx
        .read_input_registers(
            reg_address,
            register_count(&value_type)
        )
        .await
        .map_err(|e| anyhow!("Failed to fetch data: {:?}", e))??;

    interpret_modbus_register_return_type(
        &data,
        value_type,
        divide_by
    )
}

fn register_count(value_type: &ModbusRegisterType) -> u16 {
    match value_type {
        ModbusRegisterType::UINT16 => 1,
        ModbusRegisterType::UINT32 => 2,
        ModbusRegisterType::INT16 => 1,
        ModbusRegisterType::INT32 => 2,
        _ => 0,
    }
}
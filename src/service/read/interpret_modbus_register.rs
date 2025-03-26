use anyhow::{anyhow, Result};
use crate::model::modbus::modbus_register_models::ModbusRegisterType;

pub fn interpret_modbus_register_return_type(
    data: &[u16],
    value_type: ModbusRegisterType,
    divide_by: i16,
) -> Result<Option<f64>> {
    let result = match value_type {
        ModbusRegisterType::UINT16 => Some(interpret_modbus_u16(data)? / divide_by as f64),
        ModbusRegisterType::UINT32 => Some(interpret_modbus_u32(data)? / divide_by as f64),
        ModbusRegisterType::INT16 => Some(interpret_modbus_i16(data)? / divide_by as f64),
        ModbusRegisterType::INT32 => Some(interpret_modbus_i32(data)? / divide_by as f64),
        ModbusRegisterType::None => None,
    };

    Ok(result)
}

pub fn interpret_modbus_u16(data: &[u16]) -> Result<f64> {
    if data.len() == 1 {
        Ok(data[0] as f64)
    } else {
        Err(anyhow!("interpret_modbus_u16 is meant to take a &[u16; 1]"))
    }
}

pub fn interpret_modbus_u32(data: &[u16]) -> Result<f64> {
    if data.len() == 2 {
        Ok(((data[0] as u32) << 16 | data[1] as u32) as f64)
    } else {
        Err(anyhow!("interpret_modbus_u16 is meant to take a &[u16; 2]"))
    }
}

pub fn interpret_modbus_i16(data: &[u16]) -> Result<f64> {
    if data.len() == 1 {
        Ok(data[0] as i16 as f64)
    } else {
        Err(anyhow!("interpret_modbus_u16 is meant to take a &[u16; 1]"))
    }
}

pub fn interpret_modbus_i32(data: &[u16]) -> Result<f64> {
    if data.len() == 2 {
        Ok(((data[0] as i32) << 16 | data[1] as i32) as f64)
    } else {
        Err(anyhow!("interpret_modbus_u16 is meant to take a &[u16; 2]"))
    }
}

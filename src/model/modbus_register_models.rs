use serde_derive::Serialize;
use crate::model::gems_3500_memory_map_schema_models::Gems3500MemoryMap;

#[derive(Serialize, Clone, Debug)]
pub enum ModbusRegisterType {
    UINT16,
    UINT32,
    INT16,
    INT32,
    None,
}

#[derive(Serialize, Clone, Debug)]
pub struct ModbusRegister {
    pub address: u16,
    pub value_type: ModbusRegisterType,
    pub divide_by: i16,
}

impl From<Gems3500MemoryMap> for ModbusRegister {
    fn from(row: Gems3500MemoryMap) -> Self {
        ModbusRegister {
            address: row.memory_address as u16,
            value_type: match row.data_type.as_deref() {
                Some("INT16") => ModbusRegisterType::INT16,
                Some("INT32") => ModbusRegisterType::INT32,
                Some("UINT16") => ModbusRegisterType::UINT16,
                Some("UINT32") => ModbusRegisterType::UINT32,
                _ => ModbusRegisterType::None,
            },
            divide_by: row.divide_by.unwrap_or(1),
        }
    }
}
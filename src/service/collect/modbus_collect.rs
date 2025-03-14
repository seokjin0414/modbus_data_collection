use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::{anyhow, Result};
use serde_derive::Serialize;

use crate::model::{
    gems_3500_memory_map_schema_models::Gems3500MemoryMapTable,
    modbus_register_models::ModbusRegister,
};
use crate::service::read::read_from_addr::read_from_addr;

const NUMBER_OF_FLOORS: usize = 1;
const NUMBER_OF_REG_ADDRESSES: usize = 13;
const GEMS_REGISTER_ADDRESSSES: [u16; NUMBER_OF_REG_ADDRESSES] = [
    8000, 8018, 8036, 8054, 8072, 8090, 8126, 8144, 8162, 8180, 8198, 8216, 8234,
];

#[rustfmt::skip]
const GEMS_ADDR_BY_FLOOR: [SocketAddr; NUMBER_OF_FLOORS] = [
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 1, 3, 111)), 502),
];

#[derive(Serialize, Debug)]
struct PresentRes {
    results: Vec<Option<GemsData>>,
}

#[derive(Serialize, Debug)]
struct GemsData {
    floor: String,
    sums: Vec<Option<f64>>,
}

pub async fn modbus_collect() ->Result<bool> {
    let mut res_vec: Vec<Result<Vec<Option<f64>>>> = Vec::with_capacity(NUMBER_OF_FLOORS);
    let mut modbus_register_vec: Vec<ModbusRegister> = Vec::with_capacity(NUMBER_OF_REG_ADDRESSES);
    let gems_table = Gems3500MemoryMapTable::from_csv()
        .map_err(|e| anyhow!("Failed to initialize Gems3500MemoryMapTable: {:?}", e))?;

    for reg_addr in GEMS_REGISTER_ADDRESSSES {
        let gems_3500_mem_map = gems_table.rows[match
            gems_table
            .idx_memory_address
            .get(&(reg_addr as i16))
        {
            Some(idx) => *idx,
            None => {
                return Err(anyhow!("GEMS_REGISTER_ADDRESSES misconfigured; reg_addr {} invalid. Aborting.", reg_addr));
            }
        }]
            .clone();

        modbus_register_vec.push(ModbusRegister::from(gems_3500_mem_map));
    }
    let mut handles = Vec::with_capacity(NUMBER_OF_FLOORS);

    for socket_addr in GEMS_ADDR_BY_FLOOR.into_iter() {
        let modbus_register_vec_clone = modbus_register_vec.clone();
        handles.push(tokio::spawn(async move {
            read_from_addr(socket_addr, modbus_register_vec_clone).await
        }));
    }

    for handle in handles {
        res_vec.push(handle.await.unwrap());
    }

    let mut present_res = PresentRes {
        results: Vec::with_capacity(NUMBER_OF_FLOORS),
    };

    for (i, result) in res_vec.into_iter().enumerate() {
        match result {
            Ok(sums) => {
                let floor = format!("Floor {}", i + 1);
                present_res.results.push(Some(GemsData { floor, sums }));
            }
            Err(_) => {
                present_res.results.push(None);
            }
        }
    }

    // Log the transformed result
    println!(
        "Building modbus data collected: {:?}",
        serde_json::to_string(&present_res).unwrap_or("JSON serialization failed.".to_owned())
    );

    Ok(true)
}
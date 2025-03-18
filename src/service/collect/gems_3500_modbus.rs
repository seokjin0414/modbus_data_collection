use std::net::IpAddr;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::error;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::{
    model::{
        gems_3005::{
            data_models::{CollectionSet, MeasurementPoint, SetData},
            gems_3500_memory_map_models::{Gems3500MemoryMapTable, Gems3500MemoryMap},
        },
        modbus::modbus_register_models::ModbusRegister,
    },

    service::read::read_from_addr::read_from_point_map,
};

pub async fn collection_gems_3500_modbus(
) -> Result<Vec<SetData>> {
    let measurement_points = MeasurementPoint::from_csv().await
        .map_err(|e| anyhow!("Could not fetch gems.csv data: {:?}", e))?;
    let len = measurement_points.len();

    let gems_table = Gems3500MemoryMapTable::from_csv()
        .map_err(|e| anyhow!("Could not fetch gems_3500_memory_map.csv data: {:?}", e))?;

    let point_map: DashMap<(IpAddr, u16), Vec<CollectionSet>> =
        measurement_points
            .into_iter()
            .fold(DashMap::new(), |map, d| {
                let addrs = register_from_ch(d.channel);
                let mut registers = Vec::new();

                for u in addrs {
                    let map = gems_table.get_map(u)
                        .map_err(|e| anyhow!("Could not fetch gems_table: {}", e))?;
                    registers.push(ModbusRegister::from(map))
                }

                map.entry((d.host, d.port as u16))
                    .or_default()
                    .push(CollectionSet::new(d, registers));
                map
            });

    let date = Utc::now();
    let mut futures = FuturesUnordered::new();

    for (key, value) in point_map.into_iter() {
        let date = date.clone();
        let ip = key.0;
        let port = key.1;
        let data = value;

        let future = async move {
            match read_from_point_map(ip, port, data, date).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    error!("Failed to read from {}:{} - {:?}", ip, port, e);
                    Err(e)
                }
            }
        };

        futures.push(future);
    }

    let mut vec = Vec::with_capacity(len);

    while let Some(res) = futures.next().await {
        match res {
            Ok(set_data_list) => {
                vec.extend(set_data_list);
            },
            Err(_e) => {
            }
        }
    }

    Ok(vec)
}

// Hard coding (required data type)
pub fn register_from_ch (ch: u16) -> Vec<u16> {
    let addr: [u16; 18] = [
        2420+((ch-1)*64), 2420+((ch-1)*64)+2, 2420+((ch-1)*64)+4, 2420+((ch-1)*64)+10,
        2420+((ch-1)*64)+16, 2420+((ch-1)*64)+18, 2420+((ch-1)*64)+20, 2420+((ch-1)*64)+29,
        2420+((ch-1)*64)+32, 2420+((ch-1)*64)+34, 2420+((ch-1)*64)+36, 2420+((ch-1)*64)+45,
        2420+((ch-1)*64)+48, 2420+((ch-1)*64)+50, 2420+((ch-1)*64)+52, 2420+((ch-1)*64)+61,
        8000+((ch-1)*18), 9000+(ch-1)*4,
    ];

    addr.to_vec()
}
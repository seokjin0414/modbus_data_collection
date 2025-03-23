use std::net::IpAddr;
use std::time::Instant;
use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::error;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::{
    model::{
        gems_3005::{
            data_models::{CollectionSet, MeasurementPoint, SetData},
            gems_3500_memory_map_models::Gems3500MemoryMapTable,
        },
        modbus::modbus_register_models::ModbusRegister,
    },

    service::read::read_from_addr::read_from_point_map,
};

pub async fn collection_gems_3500_modbus(
) -> Result<Vec<SetData>> {
    let start = Instant::now();
    let measurement_points = MeasurementPoint::from_csv()
        .map_err(|e| anyhow!("Could not fetch gems.csv data: {:?}", e))?;
    let len = measurement_points.len();

    let gems_table = Gems3500MemoryMapTable::from_csv()
        .map_err(|e| anyhow!("Could not fetch gems_3500_memory_map.csv data: {:?}", e))?;

    let start_1 = Instant::now();
    let point_map: DashMap<(IpAddr, u16), Vec<CollectionSet>> =
        measurement_points
            .into_iter()
            .try_fold(DashMap::new(), |map, d| -> Result<DashMap<(IpAddr, u16), Vec<CollectionSet>>> {
                let addrs = register_from_ch(d.channel);
                let mut registers = Vec::new();

                for u in addrs {
                    let gems_map = gems_table.get_map(u as i16)
                        .map_err(|e| anyhow!("Could not fetch gems_table: {}", e))?;
                    registers.push(ModbusRegister::from(gems_map));
                }

                map.entry((d.host, d.port as u16))
                    .or_default()
                    .push(CollectionSet::new(d, registers));
                Ok(map)
            })?;
    println!("point_map spend time: {:?}", start_1.elapsed());

    let date = Utc::now();
    let mut futures = FuturesUnordered::new();

    let start_2 = Instant::now();
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
    println!("point_map.into_iter() spend time: {:?}", start_2.elapsed());


    let mut vec = Vec::with_capacity(len);
    let start_3 = Instant::now();
    while let Some(res) = futures.next().await {
        match res {
            Ok(set_data_list) => {
                vec.extend(set_data_list);
            },
            Err(_e) => {
            }
        }
    }
    println!("futures wait spend time: {:?}", start_3.elapsed());
    println!("spend time: {:?}", start.elapsed());
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
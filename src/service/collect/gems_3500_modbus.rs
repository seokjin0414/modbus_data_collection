use std::net::IpAddr;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::error;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::model::{
    gems_3005::{
        data_models::{MeasurementPoint, SetData},
        gems_3500_memory_map_models::Gems3500MemoryMapTable,
    },
    modbus::modbus_register_models::ModbusRegister,
};

pub async fn collection_gems_3500_modbus(
) -> Result<Vec<SetData>> {
    let measurement_point = MeasurementPoint::from_csv().await
        .map_err(|e| anyhow!("Could not fetch gems.csv data: {:?}", e))?;
    let len = measurement_point.len();

    let gems_table = Gems3500MemoryMapTable::from_csv()
        .map_err(|e| anyhow!("Could not fetch gems_3500_memory_map.csv data: {:?}", e))?;

    let point_map: DashMap<(IpAddr, u16), Vec<CollectionSet>> =
        measurement_point
            .into_iter()
            .fold(DashMap::new(), |map, d| {
                let gems_map = match gems_table.get_map(d.register) {
                    Ok(m) => m,
                    Err(_) => return Default::default(),
                };

                map.entry((d.host, d.port as u16))
                    .or_default()
                    .push(CollectionSet::new(d, ModbusRegister::from(gems_map)));
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
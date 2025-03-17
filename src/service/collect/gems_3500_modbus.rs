use std::net::IpAddr;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use chrono::Utc;
use dashmap::DashMap;
use tracing::error;
use futures::stream::{FuturesUnordered, StreamExt};
use crate::model::modbus::modbus_register_models::ModbusRegister;

// pub async fn collection_gems_3500_modbus(
// ) -> Result<Vec<SetData>> {
//     let measurement_point = get_collection_target(&state).await
//         .map_err(|e| anyhow!("Could not fetch data for collection_gems_3500_modbus: {:?}", e))?;
//
//     let len = measurement_point.len();
//     let gems_table = &state.gems3500MemoryMapTable;
//
//     let point_map: DashMap<(IpAddr, u16), Vec<CollectionSet>> =
//         measurement_point
//             .into_iter()
//             .fold(DashMap::new(), |map, d| {
//                 let gems_map = match gems_table.get_map(d.register) {
//                     Ok(m) => m,
//                     Err(_) => return Default::default(),
//                 };
//
//                 map.entry((d.host, d.port as u16))
//                     .or_default()
//                     .push(CollectionSet::new(d, ModbusRegister::from(gems_map)));
//                 map
//             });
//
//     let date = Utc::now();
//     let mut futures = FuturesUnordered::new();
//
//     for (key, value) in point_map.into_iter() {
//         let date = date.clone();
//         let ip = key.0;
//         let port = key.1;
//         let data = value;
//
//         let future = async move {
//             match read_from_point_map(ip, port, data, date).await {
//                 Ok(result) => Ok(result),
//                 Err(e) => {
//                     error!("Failed to read from {}:{} - {:?}", ip, port, e);
//                     Err(e)
//                 }
//             }
//         };
//
//         futures.push(future);
//     }
//
//     let mut vec = Vec::with_capacity(len);
//
//     while let Some(res) = futures.next().await {
//         match res {
//             Ok(set_data_list) => {
//                 vec.extend(set_data_list);
//             },
//             Err(_e) => {
//             }
//         }
//     }
//
//     //println!("@@@@@@@result: {:?}", vec);
//     Ok(vec)
// }
//
// pub async fn get_collection_target(
//     state: &Arc<ServerState>,
// ) -> Result<Vec<GetMeasurementPoint>> {
//     let client = state.pool.get().await.map_err(|e| {
//         anyhow!("Could not acquire connection from pool while get_collection_target: {:?}", e)
//     })?;
//
//     Ok(GetMeasurementPoint::from_rows(result))
// }
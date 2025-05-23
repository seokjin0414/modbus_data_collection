use anyhow::{Result, anyhow};
use dashmap::DashMap;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::error;

use crate::{
    model::{
        gems_3005::data_models::{CollectionSet, SetData},
        modbus::modbus_register_models::ModbusRegister,
    },
    service::{
        read::read_from_addr::read_from_point_map,
        server::get_state::ServerState,
        utils::create_time::{MINUTE, utc_now_ago},
    },
};

pub async fn collection_gems_3500_modbus(state: &Arc<ServerState>) -> Result<()> {
    let measurement_points = state.measurement_point.clone();
    let len = measurement_points.len();
    let gems_table = state.gems_3500_memory_map_table.clone();

    let point_map: DashMap<(IpAddr, u16, u8, bool), Vec<CollectionSet>> =
        measurement_points.into_iter().try_fold(
            DashMap::new(),
            |map, d| -> Result<DashMap<(IpAddr, u16, u8, bool), Vec<CollectionSet>>> {
                let addrs = register_from_ch(d.channel);
                let mut registers = Vec::new();

                for u in addrs {
                    let gems_map = gems_table
                        .get_map(u as i16)
                        .map_err(|e| anyhow!("Could not fetch gems_table: {}", e))?;
                    registers.push(ModbusRegister::from(gems_map));
                }

                map.entry((d.host, d.port as u16, d.unit_id, d.export_sum_status))
                    .or_default()
                    .push(CollectionSet::new(d, registers));
                Ok(map)
            },
        )?;

    let date = utc_now_ago(0, MINUTE);
    let mut futures = FuturesUnordered::new();

    for (key, value) in point_map.into_iter() {
        let date = date.clone();
        let ip = key.0;
        let port = key.1;
        let unit_id = key.2;
        let export_sum_status = key.3;
        let data = value;

        let future = async move {
            match read_from_point_map(ip, port, unit_id, export_sum_status, data, date).await {
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
    let checker = Instant::now();
    while let Some(res) = futures.next().await {
        match res {
            Ok(set_data_list) => {
                vec.extend(set_data_list);
            }
            Err(_e) => {}
        }
    }
    println!("futures wait spend time: {:?}", checker.elapsed());
    println!("{:?}", vec);

    // post_axum_server_renewal_data(vec)
    //     .await
    //     .map_err(|e| anyhow!("Request failed: {:?}", e))?;

    Ok(())
}

// Hard coding (required data type)
pub fn register_from_ch(ch: u16) -> Vec<u16> {
    let addr: [u16; 18] = [
        2420 + ((ch - 1) * 64),
        2420 + ((ch - 1) * 64) + 2,
        2420 + ((ch - 1) * 64) + 4,
        2420 + ((ch - 1) * 64) + 10,
        2420 + ((ch - 1) * 64) + 16,
        2420 + ((ch - 1) * 64) + 18,
        2420 + ((ch - 1) * 64) + 20,
        2420 + ((ch - 1) * 64) + 29,
        2420 + ((ch - 1) * 64) + 32,
        2420 + ((ch - 1) * 64) + 34,
        2420 + ((ch - 1) * 64) + 36,
        2420 + ((ch - 1) * 64) + 45,
        2420 + ((ch - 1) * 64) + 48,
        2420 + ((ch - 1) * 64) + 50,
        2420 + ((ch - 1) * 64) + 52,
        2420 + ((ch - 1) * 64) + 61,
        8000 + ((ch - 1) * 18),
        9000 + (ch - 1) * 4,
    ];

    addr.to_vec()
}

pub async fn post_axum_server_renewal_data(params: Vec<SetData>) -> Result<()> {
    let client = Client::new();

    client
        .post("")
        .json(&params)
        .send()
        .await
        .map_err(|e| anyhow!("Request failed: {:?}", e))?;
    Ok(())
}

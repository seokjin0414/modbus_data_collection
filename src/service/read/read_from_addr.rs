use super::read_from_register::read_from_register;
use crate::model::gems_3005::data_models::{GemsCollectionSet, GemsSetData, GemsSetValue};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_modbus::{Slave, client::tcp};
use tracing::error;

pub async fn read_from_point_map(
    ip: IpAddr,
    port: u16,
    unit_id: u8,
    export_sum_status: bool,
    data: Vec<GemsCollectionSet>,
    date: DateTime<Utc>,
) -> Result<Vec<GemsSetData>> {
    let addr = SocketAddr::new(ip, port);
    let ctx = match tcp::connect_slave(addr, Slave::from(unit_id)).await {
        Ok(context) => Arc::new(Mutex::new(context)),
        Err(e) => {
            error!("Could not TCP connect_slave to {}: {:?}", addr, e);
            return Ok(vec![]);
        }
    };

    let mut result = Vec::with_capacity(data.len());

    for set in data.into_iter() {
        let register_futures: Vec<_> = set
            .modbus_register
            .iter()
            .enumerate()
            .map(|(i, mr)| {
                let ctx = Arc::clone(&ctx);
                let addr_a = mr.address;
                let value_type = mr.value_type.clone();
                let divide_by = mr.divide_by;
                async move {
                    // ctx Arc<Mutex<_>> 이므로, lock 후 사용
                    let mut conn = ctx.lock().await;
                    let v = 
                        if i == 17 && !export_sum_status {
                            None
                        } else {
                            match read_from_register(&mut *conn, addr_a, value_type, divide_by).await {
                                Ok(f) => f,
                                Err(e) => {
                                    error!("Could not read from register at address: {} (register: {}): {:?}", addr, addr_a, e);
                                    None
                                }
                            }
                        };

                    Ok((i, v)) as Result<(usize, Option<f64>)>
                }
            })
            .collect();

        let register_results = join_all(register_futures).await;
        let mut values = GemsSetValue::new();

        for res in register_results {
            let (i, v) = res?;
            match i {
                0 => values.wire = v,
                1 => values.total_a = v,
                2 => values.total_w = v,
                3 => values.total_pf = v,
                4 => values.r_v = v,
                5 => values.r_a = v,
                6 => values.r_w = v,
                7 => values.r_pf = v,
                8 => values.s_v = v,
                9 => values.s_a = v,
                10 => values.s_w = v,
                11 => values.s_pf = v,
                12 => values.t_v = v,
                13 => values.t_a = v,
                14 => values.t_w = v,
                15 => values.t_pf = v,
                16 => values.kwh_sum = v,
                17 => values.kwh_export_sum = v,
                _ => {}
            }
        }

        result.push(set.to_set_data(values, date));
    }

    Ok(result)
}

use std::net::{IpAddr, SocketAddr};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use tokio_modbus::client::tcp;
use crate::model::gems_3005::data_models::{CollectionSet, SetData, SetValue};
use super::read_from_register::read_from_register;

pub async fn read_from_point_map(
    ip: IpAddr,
    port: u16,
    data: Vec<CollectionSet>,
    date: DateTime<Utc>
) -> Result<Vec<SetData>> {
    let addr = SocketAddr::new(ip, port);
    let mut ctx = tcp::connect(addr).await
        .map_err(|e| anyhow!("Could not TCP connect to {}: {:?}", addr, e))?;

    let mut result = Vec::with_capacity(data.len());

    for set in data.into_iter() {
        let mut values = SetValue::new();

        for (i, mr) in set.modbus_register.iter().enumerate() {
            let v = read_from_register(
                &mut ctx,
                mr.address,
                mr.value_type.clone(),
                mr.divide_by,
            )
                .await
                .map_err(|e| anyhow!("Could not read from register: {:?}", e))?;

            match i {
                0  => values.total_a = v,
                1  => values.total_w = v,
                2  => values.total_pf = v,
                3  => values.r_v = v,
                4  => values.r_a = v,
                5  => values.r_w = v,
                6  => values.r_pf = v,
                7  => values.s_v = v,
                8  => values.s_a = v,
                9  => values.s_w = v,
                10 => values.s_pf = v,
                11 => values.t_v = v,
                12 => values.t_a = v,
                13 => values.t_w = v,
                14 => values.t_pf = v,
                15 => values.kwh_sum = v,
                16 => values.kwh_export_sum = v,
                _ => {},
            }
        }
        result.push(set.to_set_data(values, date));
    }

    Ok(result)
}
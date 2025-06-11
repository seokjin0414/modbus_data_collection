use anyhow::{Context, Result};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};
use serde_json::to_value;
use std::{collections::HashMap, sync::Arc};
use tokio_modbus::{
    client::{Context as MContext, Reader},
    prelude::*,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    model::{gems_3005::data_models::RequestBody, heat::data_models::HeatData},
    service::{server::get_state::ServerState, utils::create_time::utc_now_minute},
};

use super::gems_3500_modbus::post_axum_server_direct_data;

pub async fn handle_heat_data(state: Arc<ServerState>) -> Result<()> {
    let measurement_points = state.heat_measurement_point.clone();

    let building_id = measurement_points[0].building_id;

    // TODO: 6층에서 가져오는 외부 ip와 port (테스트용)
    //  let ip = "220.80.128.247";
    // let port = 6003;

    let mut records: Vec<HeatData> = Vec::new();

    for row in &measurement_points {
        let socket_addr = format!("{}:{}", row.host, row.port).parse()?;
        let mut client = tcp::connect_slave(socket_addr, Slave::from(row.unit_id)).await?;

        let instant_flow = get_modbus_data(&mut client, 0x00, 2).await?;
        let instant_heat = get_modbus_data(&mut client, 0x02, 2).await?;
        let supply_temperature = get_modbus_data(&mut client, 0x20, 2).await?;
        let return_temperature = get_modbus_data(&mut client, 0x22, 2).await?;
        let cumulative_flow = get_modbus_data(&mut client, 0x70, 2).await?;
        let cumulative_heat = get_modbus_data(&mut client, 0x76, 2).await?;

        let now: DateTime<Utc> = utc_now_minute();

        println!(
            "Building: {}, Measurement Point: {}, Time: {}, Instant Flow: {}, Instant Heat: {}, Supply Temp: {}, Return Temp: {}, Cumulative Flow: {}, Cumulative Heat: {}",
            building_id,
            row.measurement_point_id,
            now,
            instant_flow,
            instant_heat,
            supply_temperature,
            return_temperature,
            cumulative_flow,
            cumulative_heat
        );

        records.push(HeatData {
            building_id,
            measurement_point_id: row.measurement_point_id,
            instant_flow: Some(instant_flow as f64),
            instant_heat: Some(instant_heat as f64),
            supply_temperature: Some(supply_temperature as f64),
            return_temperature: Some(return_temperature as f64),
            cumulative_flow: Some(cumulative_flow as f64),
            cumulative_heat: Some(cumulative_heat as f64),
            recorded_at: now,
        })
    }

    // TODO: records api 로 호출
    // 4) HTTP POST
    let params = RequestBody {
        sensor_type: HEAT.to_owned(),
        building_id,
        data: to_value(&records).context("Failed to convert records to JSON Value")?,
    };

    if let Err(e) = post_axum_server_direct_data(params).await {
        error!("Error posting IAQ data to Axum server: {:?}", e);
    } else {
        info!("Successfully posted IAQ data");
    }
    Ok(())
}

// 레지스터에서 수집한 데이터 포맷팅
fn data_format(reg: &[u16]) -> f32 {
    let high = reg[1] as u32;
    let low = reg[0] as u32;
    let combined = (high << 16) + low;
    f32::from_bits(combined)
}

async fn get_modbus_data(client: &mut MContext, start_addr: u16, length: u16) -> Result<f32> {
    let regs = client.read_holding_registers(start_addr, length).await?;
    Ok(data_format(&regs))
}

use anyhow::{Context, Result, anyhow};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};
use serde_json::to_value;
use std::sync::Arc;
use std::time::Instant;
use tokio_modbus::{
    client::{Context as ModbusContext, Reader},
    prelude::*,
};
use tracing::{error, info};

use crate::{
    model::{
        gems_3005::data_models::{HEAT, RequestBody},
        heat::data_models::HeatData,
    },
    service::{server::get_state::ServerState, utils::create_time::utc_now_minute},
};

use super::gems_3500_modbus::post_axum_server_direct_data;

pub async fn handle_heat_data(state: Arc<ServerState>) -> Result<()> {
    let start = Instant::now();

    let measurement_points = state.heat_measurement_point.clone();

    if measurement_points.is_empty() {
        info!("No heat measurement points—skipping data collection");
        return Ok(());
    }

    let building_id = measurement_points[0].building_id;

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

        // println!(
        //     "Building: {}, Measurement Point: {}, Time: {}, Instant Flow: {}, Instant Heat: {}, Supply Temp: {}, Return Temp: {}, Cumulative Flow: {}, Cumulative Heat: {}",
        //     building_id,
        //     row.measurement_point_id,
        //     now,
        //     instant_flow,
        //     instant_heat,
        //     supply_temperature,
        //     return_temperature,
        //     cumulative_flow,
        //     cumulative_heat
        // );

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

    // HTTP POST
    let params = RequestBody {
        sensor_type: HEAT.to_owned(),
        building_id,
        data: to_value(&records).context("Failed to convert records to JSON Value")?,
    };

    if let Err(e) = post_axum_server_direct_data(params).await {
        error!("Error posting HEAT data to Axum server: {:?}", e);
    } else {
        info!("Successfully posted HEAT data: {:?}", start.elapsed());
    }
    Ok(())
}

// 레지스터에서 수집한 데이터 포맷팅
fn data_format(reg: &[u16]) -> Result<f32> {
    if reg.len() != 2 {
        anyhow::bail!("Expected 2 registers, got {}", reg.len());
    }
    let mut buf = [0u8; 4];
    BigEndian::write_u16(&mut buf[0..2], reg[1]);
    BigEndian::write_u16(&mut buf[2..4], reg[0]);
    Ok(BigEndian::read_f32(&buf))
}

async fn get_modbus_data(client: &mut ModbusContext, start_addr: u16, length: u16) -> Result<f32> {
    let regs = client
        .read_holding_registers(start_addr, length)
        .await?
        .map_err(|e| anyhow!("Heat Modbus read error: {:?}", e))?;

    data_format(&regs)
}

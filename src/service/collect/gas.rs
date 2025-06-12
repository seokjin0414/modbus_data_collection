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
        gas::data_models::GasData,
        gems_3005::data_models::{GAS, RequestBody},
    },
    service::{server::get_state::ServerState, utils::create_time::utc_now_minute},
};

use super::gems_3500_modbus::post_axum_server_direct_data;

pub async fn handle_gas_data(state: Arc<ServerState>) -> Result<()> {
    let start = Instant::now();

    let measurement_points = state.gas_measurement_point.clone();

    let building_id = measurement_points[0].building_id;

    let mut records: Vec<GasData> = Vec::new();

    for row in &measurement_points {
        let socket_addr = format!("{}:{}", row.host, row.port).parse()?;

        let mut client = tcp::connect_slave(socket_addr, Slave::from(row.unit_id)).await?;

        // 적산유량 (m³)
        let reg = client
            .read_input_registers(0x00, 2)
            .await?
            .context("Failed to read cumulative flow registers")?;
        let cumulative_flow = (reg[0] as u32) * 10_000 + (reg[1] as u32);

        // 순시유량 (m³/h)
        let reg = client
            .read_input_registers(0x02, 2)
            .await?
            .context("Failed to read instant flow registers")?;
        let instant_flow = (reg[0] as f64) * 100.0 + (reg[1] as f64) / 100.0;

        // 압력 (kPa)
        let reg = client
            .read_input_registers(0x08, 2)
            .await?
            .context("Failed to read pressure registers")?;

        let pressure = (reg[0] as f64) * 100.0 + (reg[1] as f64) / 100.0;

        // 온도 (°C)
        let reg = client
            .read_input_registers(0x0A, 2)
            .await?
            .context("Failed to read temperature registers")?;

        let temp = if reg[0] == 0 {
            reg[1] as f64 / 100.0
        } else {
            -(reg[1] as f64) / 100.0
        };

        // 연결 종료는 drop으로 자동 처리됩니다
        drop(client);

        let now: DateTime<Utc> = utc_now_minute();

        println!(
            "Building: {}, Measurement Point: {}, Time: {}, Instant Flow: {},  Cumulative Flow: {}, Pressure: {}, Temp: {}",
            building_id,
            row.measurement_point_id,
            now,
            instant_flow,
            cumulative_flow,
            pressure,
            temp
        );

        records.push(GasData {
            building_id,
            measurement_point_id: row.measurement_point_id,
            recorded_at: now,
            instant_flow: Some(instant_flow as f64),
            cumulative_flow: Some(cumulative_flow as f64),
            pressure: Some(pressure as f64),
            temp: Some(temp as f64),
        })
    }

    // HTTP POST
    let params = RequestBody {
        sensor_type: GAS.to_owned(),
        building_id,
        data: to_value(&records).context("Failed to convert records to JSON Value")?,
    };

    if let Err(e) = post_axum_server_direct_data(params).await {
        error!("Error posting GAS data to Axum server: {:?}", e);
    } else {
        info!("Successfully posted GAS data: {:?}", start.elapsed());
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

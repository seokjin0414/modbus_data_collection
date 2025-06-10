use crate::{
    model::{
        gems_3005::data_models::{IAQ, RequestBody},
        iaq::data_models::{CcmData, Header, IaqData},
    },
    service::{
        collect::gems_3500_modbus::post_axum_server_direct_data, server::get_state::ServerState,
        utils::create_time::utc_now_minute,
    },
};
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt};
use serde_json::to_value;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
};
use tracing::{error, info, warn};
use uuid::Uuid;

// 바이트 단위 읽기 유틸
pub fn read_u8(cur: &mut Cursor<&[u8]>) -> Result<u8> {
    cur.read_u8().context("Failed to read u8")
}

pub fn read_u16(cur: &mut Cursor<&[u8]>) -> Result<u16> {
    cur.read_u16::<BigEndian>()
        .context("Failed to read u16 big-endian")
}

pub fn read_bytes(cur: &mut Cursor<&[u8]>, n: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    cur.read_exact(&mut buf)
        .context(format!("Failed to read {} bytes", n))?;
    Ok(buf)
}

pub fn read_str_n(cur: &mut Cursor<&[u8]>, n: usize) -> Result<String> {
    let raw = read_bytes(cur, n)?;
    let trimmed: Vec<u8> = raw.into_iter().take_while(|&b| b != 0x00).collect();
    String::from_utf8(trimmed).context("Failed to decode UTF-8 string")
}

// 체크섬 검증 (헤더)
pub fn valid_checksum(hdr: &Header) -> bool {
    let mut acc: u32 = 0;
    acc += (hdr.tid >> 8) as u32 + (hdr.tid & 0xFF) as u32;
    acc += hdr.src as u32 + hdr.dst as u32;
    acc += (hdr.data_length >> 8) as u32 + (hdr.data_length & 0xFF) as u32;
    ((acc & 0xFF) as u8) == hdr.checksum
}

// 펑션코드 검증 (0x24)
pub fn valid_function_code(code: u8) -> bool {
    code == 0x24
}

// MAC 바이트 배열 → "AA:BB:CC:..." 문자열
pub fn format_mac_upper(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

// 실내환경 데이터 가공
pub fn aqm_data(registers: &[u16]) -> Result<HashMap<String, f64>> {
    if registers.len() != 64 {
        anyhow::bail!("Expected 64 registers, got {}", registers.len());
    }
    let mut m = HashMap::new();
    m.insert("temperature".to_string(), registers[0] as f64 / 10.0);
    m.insert("humidity".to_string(), registers[1] as f64 / 10.0);
    m.insert("co2".to_string(), registers[2] as f64);
    m.insert("pm25".to_string(), registers[3] as f64);
    m.insert("pm10".to_string(), registers[4] as f64);
    m.insert("tvoc".to_string(), registers[5] as f64);
    m.insert("lux".to_string(), registers[6] as f64);
    Ok(m)
}

// CCM 처리 로직 호출 (예: smart plug 데이터 저장)
pub fn ccm_data(registers: &[u16]) -> Result<CcmData> {
    if registers.len() != 64 {
        anyhow::bail!("Expected 64 registers, got {}", registers.len());
    }

    Ok(CcmData {
        onoff: registers[0],
        voltage: registers[1] as f64 / 100.0,
        current: registers[2] as f64 / 1000.0,
        watt: registers[3] as f64 / 10.0,
        power_factor: registers[4] as f64 / 10.0,
        today_usage: registers[5] as f64 / 10.0,
        this_month_usage: (registers[26] as u32) + ((registers[27] as u32) << 16),
    })
}

// 실제 IAQ 처리 로직 호출 (예: 상태에 버퍼 쌓기 / API 전송 등)
pub async fn handle_iaq(state: Arc<ServerState>, mac: String, registers: Vec<u16>) -> Result<()> {
    // 1) 레지스터 → 값 맵
    let data_map = aqm_data(&registers).context("Failed to convert IAQ registers to data map")?;

    // 2) MAC으로 매핑된 IAQ 포인트 조회
    let mappings: Vec<_> = state
        .iaq_measurement_point
        .iter()
        .filter(|mp| mp.mac.eq_ignore_ascii_case(&mac))
        .collect();

    if mappings.is_empty() {
        warn!("No IAQ measurement points found for MAC {}", mac);
        return Ok(());
    }

    // 3) 페이로드 생성
    let now = utc_now_minute();
    let mut map: HashMap<Uuid, IaqData> = HashMap::new();

    let building_id = mappings[0].building_id; // building_id는 모두 동일

    for mp in mappings {
        if let Some(&value) = data_map.get(&mp.iaq_type) {
            map.insert(
                mp.measurement_point_id,
                IaqData {
                    building_id: mp.building_id,
                    measurement_point_id: mp.measurement_point_id,
                    recorded_at: now,
                    value: Some(value),
                },
            );
        }
    }

    if map.is_empty() {
        info!("No matching data types for MAC {} → skipping API call", mac);
        return Ok(());
    }
    let records: Vec<IaqData> = map.into_iter().map(|(_, iaqdata)| iaqdata).collect();

    // 4) HTTP POST
    let params = RequestBody {
        sensor_type: IAQ.to_owned(),
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

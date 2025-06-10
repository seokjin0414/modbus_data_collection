use crate::model::iaq::data_models::{CcmData, Header};
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

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

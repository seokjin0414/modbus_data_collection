use crate::model::iaq::data_models::Header;
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

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
fn format_mac_upper(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

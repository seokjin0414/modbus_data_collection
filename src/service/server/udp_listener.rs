use crate::{
    model::iaq::data_models::{Header, Message},
    service::{
        read::iaq::util_funcs::{
            read_bytes, read_str_n, read_u8, read_u16, valid_checksum, valid_function_code,
        },
        server::get_state::ServerState,
    },
};
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{net::UdpSocket, sync::Mutex};
use tracing::{error, info};

// MAC 바이트 배열 → "AA:BB:CC:" 문자열
fn format_mac_upper(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

// 실제 IAQ 처리 로직 호출 (예: 상태에 버퍼 쌓기 / API 전송 등)
async fn handle_iaq(state: Arc<ServerState>, mac: String, registers: Vec<u16>) -> Result<()> {
    // TODO: state 내부의 IAQ 메모리 맵 / measurement_point lookup
    // 예: let mapping = state.iaq_map.get(&mac)?;
    tracing::info!(%mac, regs = registers.len(), "Received IAQ data");
    // aqm_data 변환, 버퍼링, API flush 로직 호출
    Ok(())
}

// 실제 CCM 처리 로직 호출 (예: smart plug 데이터 저장)
async fn handle_ccm(_state: Arc<ServerState>, mac: String, registers: Vec<u16>) -> Result<()> {
    tracing::info!(%mac, regs = registers.len(), "Received CCM data");
    // ccm_data 변환 후 저장/전송
    Ok(())
}

// UDP 리스너: 5005 포트로 들어오는 패킷 파싱 & 처리
pub async fn run_udp_listener(state: Arc<ServerState>) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5005").await?;
    info!("UDP listener bound to 0.0.0.0:5005");

    let mut buf = [0u8; 1024];
    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        let mut cur = Cursor::new(data);

        // 1) 헤더 파싱
        let hdr = Header {
            tid: read_u16(&mut cur)?,
            src: read_u8(&mut cur)?,
            dst: read_u8(&mut cur)?,
            data_length: read_u16(&mut cur)?,
            checksum: read_u8(&mut cur)?,
        };
        if !valid_checksum(&hdr) {
            error!(?hdr, "Invalid header checksum from {}", peer);
            continue;
        }

        // 2) 펑션코드
        let func = read_u8(&mut cur)?;
        if !valid_function_code(func) {
            error!(code = func, "Invalid function code from {}", peer);
            continue;
        }

        // 3) 메타 정보
        let local_addr = {
            let b = read_bytes(&mut cur, 6)?;
            let mut arr = [0u8; 6];
            arr.copy_from_slice(&b);
            arr
        };
        let ssid = read_str_n(&mut cur, 32)?;
        let mac_bytes = {
            let b = read_bytes(&mut cur, 6)?;
            let mut arr = [0u8; 6];
            arr.copy_from_slice(&b);
            arr
        };
        let device_type = read_u8(&mut cur)?;
        let cfg = read_u8(&mut cur)?;
        let mac_str = format_mac_upper(&mac_bytes);

        // 4) 메시지
        let msg = Message {
            version: read_u16(&mut cur)?,
            count: read_u8(&mut cur)?,
            offset: read_u16(&mut cur)?,
            registers: (0..read_u8(&mut cur)?)
                .map(|_| read_u16(&mut cur).unwrap())
                .collect(),
            checksum: read_u16(&mut cur)?,
        };

        // 5) 타입별 분기 처리
        match device_type {
            12 => {
                // IAQ 센서
                if let Err(e) = handle_iaq(state.clone(), mac_str, msg.registers).await {
                    error!(error = ?e, "Error handling IAQ packet");
                }
            }
            5 => {
                // 스마트콘센트
                if let Err(e) = handle_ccm(state.clone(), mac_str, msg.registers).await {
                    error!(error = ?e, "Error handling CCM packet");
                }
            }
            other => {
                error!(dtype = other, "Unknown device type from {}", peer);
            }
        }
    }
}

use crate::{
    model::iaq::data_models::{Header, Message},
    service::{
        read::iaq::util_funcs::{
            ccm_data, format_mac_upper, handle_iaq, read_bytes, read_str_n, read_u8, read_u16,
            valid_checksum, valid_function_code,
        },
        server::get_state::ServerState,
    },
};
use anyhow::Result;

use std::io::Cursor;

use std::sync::Arc;
use tokio::{
    net::UdpSocket,
    time::{Duration, timeout},
};
use tracing::{error, info, warn};

// UDP 리스너: 5005 포트로 들어오는 패킷 파싱 & 처리
pub async fn run_udp_listener(state: Arc<ServerState>) -> Result<()> {
    let listen_future = async {
        let socket = UdpSocket::bind("0.0.0.0:5005").await?;
        info!("UDP listener bound to 0.0.0.0:5005");

        let mut buf = [0u8; 1024];
        loop {
            let (len, peer) = socket.recv_from(&mut buf).await?;
            // println!("{}바이트 패킷 수신: {:?} from {}", len, &buf[..len], peer);
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

            // 3) 메타 정보 -> device_type, mac_str만 필요하나 바이트 소모는 필요해서 나머지 변수들 남겨놓음
            let _local_addr = {
                let b = read_bytes(&mut cur, 6)?;
                let mut arr = [0u8; 6];
                arr.copy_from_slice(&b);
                arr
            };
            let _ssid = read_str_n(&mut cur, 32)?;

            let mac_bytes = {
                let b = read_bytes(&mut cur, 6)?;
                let mut arr = [0u8; 6];
                arr.copy_from_slice(&b);
                arr
            };

            let device_type = read_u8(&mut cur)?;
            let _cfg = read_u8(&mut cur)?;

            let mac_str = format_mac_upper(&mac_bytes);

            // 4) 메시지
            let version = read_u16(&mut cur)?;
            let number_of_reg = read_u8(&mut cur)?; // 한 번만 읽기!
            let offset = read_u16(&mut cur)?;

            let registers: Vec<u16> = (0..number_of_reg)
                .map(|_| read_u16(&mut cur).unwrap())
                .collect();

            let checksum = read_u16(&mut cur)?;

            let msg = Message {
                version,
                count: number_of_reg,
                offset,
                registers,
                checksum,
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
                    if let Err(e) = ccm_data(&msg.registers) {
                        error!(error = ?e, "Error handling CCM packet");
                    }
                }
                other => {
                    warn!(dtype = other, "Unknown device type from {}", peer);
                }
            }
        }
    };
    match timeout(Duration::from_secs(30), listen_future).await {
        Ok(inner_res) => inner_res, // 30초 안에 에러가 나면 그 에러를 그대로 리턴
        Err(_) => {
            info!("UDP listener timed out after 30s");
            Ok(())
        }
    }
}

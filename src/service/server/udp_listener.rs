use crate::{
    model::{
        gems_3005::data_models::{IAQ, RequestBody},
        iaq::data_models::{Header, IaqData, Message},
    },
    service::{
        collect::gems_3500_modbus::post_axum_server_direct_data,
        read::iaq::util_funcs::{
            aqm_data, ccm_data, format_mac_upper, read_bytes, read_str_n, read_u8, read_u16,
            valid_checksum, valid_function_code,
        },
        server::get_state::ServerState,
        utils::create_time::utc_now_minute,
    },
};
use anyhow::{Context, Result};

use serde_json::to_value;
use std::{collections::HashMap, io::Cursor};
use uuid::Uuid;

use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{error, info, warn};

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

    info!("레코드: {:?}", records);

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

// UDP 리스너: 5005 포트로 들어오는 패킷 파싱 & 처리
pub async fn run_udp_listener(state: Arc<ServerState>) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:5005").await?;
    info!("UDP listener bound to 0.0.0.0:5005");

    let mut buf = [0u8; 1024];
    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;
        println!("{}바이트 패킷 수신: {:?} from {}", len, &buf[..len], peer);
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
        let _cfg = read_u8(&mut cur)?;

        let mac_bytes = {
            let b = read_bytes(&mut cur, 6)?;
            let mut arr = [0u8; 6];
            arr.copy_from_slice(&b);
            arr
        };
        let device_type = read_u8(&mut cur)?;
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
                if let Err(e) = ccm_data(&msg.registers) {
                    error!(error = ?e, "Error handling CCM packet");
                }
            }
            other => {
                error!(dtype = other, "Unknown device type from {}", peer);
            }
        }
    }
}

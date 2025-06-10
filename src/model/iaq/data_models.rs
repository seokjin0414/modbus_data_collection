use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct IaqMeasurementPoint {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub ip_to: IpAddr,
    pub port_to: i32,
    pub mac: String,
    pub iaq_type: String,
}

impl IaqMeasurementPoint {
    pub fn from_csv() -> Result<Vec<IaqMeasurementPoint>> {
        let mut rdr = csv::Reader::from_path("src/files/iaq.csv")?;

        let mut vec: Vec<IaqMeasurementPoint> = Vec::new();
        for result in rdr.deserialize() {
            let record: IaqMeasurementPoint = result?;
            vec.push(record);
        }

        Ok(vec)
    }
}

// UDP 패킷 헤더 구조체
#[derive(Debug)]
pub struct Header {
    pub tid: u16,
    pub src: u8,
    pub dst: u8,
    pub data_length: u16,
    pub checksum: u8,
}

// 패킷 메시지 본문
#[derive(Debug)]
pub struct Message {
    pub version: u16,
    pub count: u8,
    pub offset: u16,
    pub registers: Vec<u16>,
    pub checksum: u16,
}

#[derive(Debug)]
pub struct CcmData {
    pub onoff: u16,
    pub voltage: f64,
    pub current: f64,
    pub watt: f64,
    pub power_factor: f64,
    pub today_usage: f64,
    pub this_month_usage: u32,
}

// api 호출시 전달값
#[derive(Serialize, Debug)]
pub struct IaqData {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub value: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

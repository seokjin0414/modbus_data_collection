use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::{io, net::IpAddr};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct HeatMeasurementPoint {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub host: IpAddr,
    pub port: i32,
    pub unit_id: u8,
}

impl HeatMeasurementPoint {
    pub fn from_csv() -> Result<Vec<HeatMeasurementPoint>> {
        let mut rdr = match csv::Reader::from_path("src/files/heat.csv") {
            Ok(rdr) => rdr,
            // 파일이 없으면 빈 벡터로 처리
            Err(e) => {
                if let csv::ErrorKind::Io(io_err) = e.kind() {
                    if io_err.kind() == io::ErrorKind::NotFound {
                        return Ok(Vec::new());
                    }
                }

                return Err(e.into());
            }
        };

        let mut vec: Vec<HeatMeasurementPoint> = Vec::new();
        for result in rdr.deserialize() {
            let record: HeatMeasurementPoint = result?;
            vec.push(record);
        }

        Ok(vec)
    }
}

// api 호출시 전달값
#[derive(Serialize, Debug)]
pub struct HeatData {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub instant_flow: Option<f64>,       // 순시유량
    pub instant_heat: Option<f64>,       // 순시열량
    pub supply_temperature: Option<f64>, // 공급온도
    pub return_temperature: Option<f64>, // 환수온도
    pub cumulative_flow: Option<f64>,    // 적산유량
    pub cumulative_heat: Option<f64>,    // 적산열량
    pub recorded_at: DateTime<Utc>,
}

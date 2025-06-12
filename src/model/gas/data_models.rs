use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct GasMeasurementPoint {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub host: IpAddr,
    pub port: i32,
    pub unit_id: u8,
}

impl GasMeasurementPoint {
    pub fn from_csv() -> Result<Vec<GasMeasurementPoint>> {
        let mut rdr = csv::Reader::from_path("src/files/gas.csv")?;

        let mut vec: Vec<GasMeasurementPoint> = Vec::new();
        for result in rdr.deserialize() {
            let record: GasMeasurementPoint = result?;
            vec.push(record);
        }

        Ok(vec)
    }
}

// api 호출시 전달값
#[derive(Serialize, Debug)]
pub struct GasData {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub cumulative_flow: Option<f64>, // 적산유량
    pub instant_flow: Option<f64>,    // 순시유량
    pub pressure: Option<f64>,        // 압력
    pub temp: Option<f64>,            // 온도
    pub recorded_at: DateTime<Utc>,
}

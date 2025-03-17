use std::net::IpAddr;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_derive::Deserialize;
use uuid::Uuid;

use crate::model::modbus::modbus_register_models::ModbusRegister;

pub const GEMS_3500_MODBUS: i32 = 43;
pub const IAQ_MODBUS: i32 = 44;
pub const ENERGY: &str = "energy";
pub const RENEWABLE: &str = "renewable";
pub const UNDEFINED: &str = "undefined";

#[derive(Deserialize, Debug, Clone)]
pub struct MeasurementPoint {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub host: IpAddr,
    pub port: i32,
    pub channel: i16,
}

impl MeasurementPoint {
    pub fn from_csv() -> Result<Vec<MeasurementPoint>> {
        let mut rdr = csv::Reader::from_path("src/files/gems.csv")?;

        let mut vec: Vec<MeasurementPoint> = Vec::new();
        for result in rdr.deserialize() {
            let record: MeasurementPoint = result?;
            vec.push(record);
        }

        Ok(vec)
    }
}

#[derive(Debug)]
pub struct CollectionSet {
    pub measurement_point_id: Uuid,
    pub building_id: Uuid,
    pub host: IpAddr,
    pub port: i32,
    pub channel: i16,
    pub modbus_register: ModbusRegister,
}

impl CollectionSet {
    pub fn new(
        point: MeasurementPoint,
        modbus: ModbusRegister,
    ) -> Self {
        CollectionSet{
            measurement_point_id: point.measurement_point_id,
            building_id: point.building_id,
            host: point.host,
            port: point.port,
            channel: point.channel,
            modbus_register: modbus,
        }
    }

    pub fn to_set_data(&self, value: Option<f64>, recorded_at: DateTime<Utc>) -> SetData {
        SetData {
            building_id: self.building_id,
            measurement_point_id: self.measurement_point_id,
            value,
            recorded_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetData {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub value: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}
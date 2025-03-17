use std::net::IpAddr;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_derive::Deserialize;
use uuid::Uuid;

use crate::model::modbus::modbus_register_models::ModbusRegister;

pub const GEMS_3500_MODBUS: i32 = 43;
pub const IAQ_MODBUS: i32 = 44;

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

    pub fn to_set_data(&self, recorded_at: DateTime<Utc>) -> SetData {
        SetData {
            building_id: self.building_id,
            measurement_point_id: self.measurement_point_id,
            total_a: None,
            total_w: None,
            total_pf: None,
            r_v: None,
            r_a: None,
            r_w: None,
            r_pf: None,
            s_v: None,
            s_a: None,
            s_w: None,
            s_pf: None,
            t_v: None,
            t_a: None,
            t_w: None,
            t_pf: None,
            kwh_sum: None,
            kwh_export_sum: None,
            recorded_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetData {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub total_a: Option<f64>,
    pub total_w: Option<f64>,
    pub total_pf: Option<f64>,
    pub r_v: Option<f64>,
    pub r_a: Option<f64>,
    pub r_w: Option<f64>,
    pub r_pf: Option<f64>,
    pub s_v: Option<f64>,
    pub s_a: Option<f64>,
    pub s_w: Option<f64>,
    pub s_pf: Option<f64>,
    pub t_v: Option<f64>,
    pub t_a: Option<f64>,
    pub t_w: Option<f64>,
    pub t_pf: Option<f64>,
    pub kwh_sum: Option<f64>,
    pub kwh_export_sum: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}
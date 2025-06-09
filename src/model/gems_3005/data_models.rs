use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use crate::model::modbus::modbus_register_models::ModbusRegister;

pub const GEMS: &str = "gems";
pub const IAQ: &str = "iaq";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestBody {
    pub sensor_type: String,
    pub building_id: Uuid,
    pub data: serde_json::Value,
}

impl RequestBody {
    pub fn from_data<T>(sensor_type: &str, building_id: Uuid, data: Vec<T>) -> Result<Self> 
    where T: serde::Serialize,
    {
        let json_data = serde_json::to_value(data)
            .map_err(|e| anyhow!("Failed to convert data to json: {}", e))?;

        Ok(RequestBody {
            sensor_type: sensor_type.to_string(),
            building_id,
            data: json_data,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GemsMeasurementPoint {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub host: IpAddr,
    pub port: i32,
    pub unit_id: u8,
    pub channel: u16,
    pub export_sum_status: bool,
}

impl GemsMeasurementPoint {
    pub fn from_csv() -> Result<Vec<GemsMeasurementPoint>> {
        let mut rdr = csv::Reader::from_path("src/files/gems.csv")?;

        let mut vec: Vec<GemsMeasurementPoint> = Vec::new();
        for result in rdr.deserialize() {
            let record: GemsMeasurementPoint = result?;
            vec.push(record);
        }

        Ok(vec)
    }
}

pub struct GemsCollectionSet {
    pub measurement_point_id: Uuid,
    pub building_id: Uuid,
    pub modbus_register: Vec<ModbusRegister>,
}

impl GemsCollectionSet {
    pub fn new(point: GemsMeasurementPoint, registers: Vec<ModbusRegister>) -> Self {
        GemsCollectionSet {
            measurement_point_id: point.measurement_point_id,
            building_id: point.building_id,
            modbus_register: registers,
        }
    }

    pub fn to_set_data(&self, values: GemsSetValue, recorded_at: DateTime<Utc>) -> GemsSetData {
        GemsSetData {
            building_id: self.building_id,
            measurement_point_id: self.measurement_point_id,
            wire: values.wire,
            total_a: values.total_a,
            total_w: values.total_w,
            total_pf: values.total_pf,
            r_v: values.r_v,
            r_a: values.r_a,
            r_w: values.r_w,
            r_pf: values.r_pf,
            s_v: values.s_v,
            s_a: values.s_a,
            s_w: values.s_w,
            s_pf: values.s_pf,
            t_v: values.t_v,
            t_a: values.t_a,
            t_w: values.t_w,
            t_pf: values.t_pf,
            kwh_sum: values.kwh_sum,
            kwh_export_sum: values.kwh_export_sum,
            recorded_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GemsSetData {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub wire: Option<f64>,
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

pub struct GemsSetValue {
    pub wire: Option<f64>,
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
}

impl GemsSetValue {
    pub fn new() -> Self {
        GemsSetValue {
            wire: None,
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
        }
    }
}

use std::net::IpAddr;
use std::time::Instant;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use uuid::Uuid;

use crate::model::modbus::modbus_register_models::ModbusRegister;

#[derive(Deserialize, Debug, Clone)]
pub struct MeasurementPoint {
    pub building_id: Uuid,
    pub measurement_point_id: Uuid,
    pub host: IpAddr,
    pub port: i32,
    pub channel: u16,
}

impl MeasurementPoint {
    pub fn from_csv() -> Result<Vec<MeasurementPoint>> {
        let start = Instant::now();
        let mut rdr = csv::Reader::from_path("src/files/gems.csv")?;

        let mut vec: Vec<MeasurementPoint> = Vec::new();
        for result in rdr.deserialize() {
            let record: MeasurementPoint = result?;
            vec.push(record);
        }

        println!("MeasurementPoint from_csv spend time: {:?}", start.elapsed());
        Ok(vec)
    }
}

pub struct CollectionSet {
    pub measurement_point_id: Uuid,
    pub building_id: Uuid,
    pub modbus_register: Vec<ModbusRegister>,
}

impl CollectionSet {
    pub fn new(point: MeasurementPoint, registers: Vec<ModbusRegister>) -> Self {
        CollectionSet{
            measurement_point_id: point.measurement_point_id,
            building_id: point.building_id,
            modbus_register: registers,
        }
    }

    pub fn to_set_data(&self, values: SetValue, recorded_at: DateTime<Utc>) -> SetData {
        SetData {
            building_id: self.building_id,
            measurement_point_id: self.measurement_point_id,
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

pub struct SetValue {
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

impl SetValue {
    pub fn new() -> Self {
        SetValue {
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
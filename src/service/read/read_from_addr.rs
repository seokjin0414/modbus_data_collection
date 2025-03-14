use std::net::SocketAddr;
use anyhow::{anyhow, Result};
use tokio_modbus::client::tcp;
use tracing::error;
use crate::model::modbus::modbus_register_models::ModbusRegister;
use super::read_from_register::read_from_register;

pub async fn read_from_addr(
    addr: SocketAddr,
    reg_addresses: Vec<ModbusRegister>,
) -> Result<Vec<Option<f64>>> {
    let mut res_vec: Vec<Option<f64>> = Vec::with_capacity(reg_addresses.len());
    let mut ctx = tcp::connect(addr).await
        .map_err(|e| anyhow!("Could not TCP connect to {}: {:?}", addr, e))?;

    for reg_addr in reg_addresses {
        res_vec.push(
            match read_from_register(
                &mut ctx,
                reg_addr.address,
                reg_addr.value_type,
                reg_addr.divide_by,
            ).await {
                Ok(res) => res,
                Err(e) => {
                    error!("Could not read from register {} at {}: {:?}", reg_addr.address, addr, e);
                    None
                }
            }
        );
    }

    Ok(res_vec)
}
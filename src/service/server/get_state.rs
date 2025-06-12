use crate::model::{
    gas::data_models::GasMeasurementPoint,
    gems_3005::{
        data_models::GemsMeasurementPoint, gems_3500_memory_map_models::Gems3500MemoryMapTable,
    },
    heat::data_models::HeatMeasurementPoint,
    iaq::data_models::IaqMeasurementPoint,
};
use anyhow::{Result, anyhow};
use tokio::try_join;

pub struct ServerState {
    pub gems_3500_memory_map_table: Gems3500MemoryMapTable,
    pub gems_measurement_point: Vec<GemsMeasurementPoint>,
    pub iaq_measurement_point: Vec<IaqMeasurementPoint>,
    pub heat_measurement_point: Vec<HeatMeasurementPoint>,
    pub gas_measurement_point: Vec<GasMeasurementPoint>,
}

// 이 함수에서 서버 초기화할때 초기 state를 제공. LUT(Lookup Table)/캐시 역할을 한다.
// Inititalize the state here when the server initializes. Many of the fields here will act as caches in the form of lookup tables.
pub async fn get_state() -> Result<ServerState> {
    let gems_3500_memory_map_table = tokio::spawn(async { Gems3500MemoryMapTable::from_csv() });

    let gems_measurement_point = tokio::spawn(async { GemsMeasurementPoint::from_csv() });

    let iaq_measurement_point = tokio::spawn(async { IaqMeasurementPoint::from_csv() });

    let heat_measurement_point = tokio::spawn(async { HeatMeasurementPoint::from_csv() });

    let gas_measurement_point = tokio::spawn(async { GasMeasurementPoint::from_csv() });

    let results = try_join!(
        gems_3500_memory_map_table,
        gems_measurement_point,
        iaq_measurement_point,
        heat_measurement_point,
        gas_measurement_point
    );

    match results {
        Ok(res_tup) => {
            let gems_3500_memory_map_table = match res_tup.0 {
                Ok(mmap) => mmap,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing Gems3500MemoryMap for ServerState: {:?}",
                        e
                    ));
                }
            };

            let gems_measurement_point = match res_tup.1 {
                Ok(bdt) => bdt,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing GemsMeasurementPoint for ServerState: {:?}",
                        e
                    ));
                }
            };

            let iaq_measurement_point = match res_tup.2 {
                Ok(iaqs) => iaqs,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing IaqMeasurementPoint for ServerState: {:?}",
                        e
                    ));
                }
            };

            let heat_measurement_point = match res_tup.3 {
                Ok(heats) => heats,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing HeatMeasurementPoint for ServerState: {:?}",
                        e
                    ));
                }
            };

            let gas_measurement_point = match res_tup.4 {
                Ok(gas) => gas,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing GasMeasurementPoint for ServerState: {:?}",
                        e
                    ));
                }
            };

            Ok(ServerState {
                gems_3500_memory_map_table,
                gems_measurement_point,
                iaq_measurement_point,
                heat_measurement_point,
                gas_measurement_point,
            })
        }
        Err(e) => Err(anyhow!("JoinError while constructing ServerState: {:?}", e)),
    }
}

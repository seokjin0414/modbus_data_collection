use anyhow::{anyhow, Result};
use tokio::try_join;
use crate::model::gems_3005::{
    data_models::MeasurementPoint,
    gems_3500_memory_map_models::Gems3500MemoryMapTable,
};

pub struct ServerState {
    pub gems_3500_memory_map_table: Gems3500MemoryMapTable,
    pub measurement_point: Vec<MeasurementPoint>,
}

// 이 함수에서 서버 초기화할때 초기 state를 제공. LUT(Lookup Table)/캐시 역할을 한다.
// Inititalize the state here when the server initializes. Many of the fields here will act as caches in the form of lookup tables.
pub async fn get_state(
) -> Result<ServerState> {
    let gems_3500_memory_map_table =
        tokio::spawn(async { Gems3500MemoryMapTable::from_csv() });
    
    let measurement_point =
        tokio::spawn(async { MeasurementPoint::from_csv() });

    let results = try_join!(
        gems_3500_memory_map_table,
        measurement_point,
    );

    match results {
        Ok(res_tup) => {
            let gems_3500_memory_map_table = match res_tup.0 {
                Ok(mmap) => mmap,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing Gems3500MemoryMap for ServerState: {:?}",
                        e
                    ))
                }
            };

            let measurement_point = match res_tup.1 {
                Ok(bdt) => bdt,
                Err(e) => {
                    return Err(anyhow!(
                        "Error while constructing MeasurementPoint for ServerState: {:?}",
                        e
                    ))
                }
            };

            Ok(ServerState {
                gems_3500_memory_map_table,
                measurement_point,
            })
        }
        Err(e) => Err(anyhow!("JoinError while constructing ServerState: {:?}", e)),
    }
}
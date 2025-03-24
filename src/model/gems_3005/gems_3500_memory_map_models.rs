use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde_derive::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Gems3500MemoryMap {
    pub memory_address: i16,
    pub data_category: Option<String>,
    pub phase: Option<String>,
    pub fc: Option<i16>,
    pub size_in_bytes: Option<i16>,
    pub data_type: Option<String>,
    pub divide_by: Option<i16>,
}

#[derive(Clone)]
pub struct Gems3500MemoryMapTable {
    pub rows: Vec<Gems3500MemoryMap>,
    pub idx_memory_address: DashMap<i16, usize>,
}

impl From<Vec<Gems3500MemoryMap>> for Gems3500MemoryMapTable {
    fn from(rows: Vec<Gems3500MemoryMap>) -> Self {
        let idx_memory_address: DashMap<i16, usize> = DashMap::with_capacity(rows.len());

        for (idx, mmap) in rows.iter().enumerate() {
            idx_memory_address.insert(mmap.memory_address, idx);
        }

        Self {
            rows,
            idx_memory_address,
        }
    }
}

impl Gems3500MemoryMapTable {
    pub fn from_csv() -> Result<Gems3500MemoryMapTable> {
        let mut rdr = csv::Reader::from_path("src/files/gems_3500_memory_map.csv")?;

        let mut maps: Vec<Gems3500MemoryMap> = Vec::new();
        for result in rdr.deserialize() {
            let record: Gems3500MemoryMap = result?;
            maps.push(record);
        }

        let idx_memory_address: DashMap<i16, usize> = DashMap::with_capacity(maps.len());
        for (idx, mmap) in maps.iter().enumerate() {
            idx_memory_address.insert(mmap.memory_address, idx);
        }

        Ok(Self {
            rows: maps,
            idx_memory_address,
        })
    }

    pub fn get_map(&self, register: i16) -> Result<Gems3500MemoryMap> {
        let idx = self
            .idx_memory_address
            .get(&register)
            .ok_or_else(||
                anyhow!("GEMS_REGISTER_ADDRESSES misconfigured; reg_addr {} invalid. Aborting.", register)
            )?;

        Ok(self.rows[*idx].clone())
    }
}
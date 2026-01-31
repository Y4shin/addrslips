mod model;
mod util;
mod address;
pub use model::*;
use serde::{Deserialize, Serialize};

use std::{
    io::{Error, ErrorKind}, path::PathBuf
};

use rmp_serde::{from_slice, to_vec};

use address::AddressDatabase;


#[derive(Serialize, Deserialize)]
struct DatabaseDump {
    addresses: Vec<Address>,
}

pub struct ProjectDatabase {
    db_file: PathBuf,
    addresses: AddressDatabase,
}


impl ProjectDatabase {
    pub fn new(db_file: PathBuf) -> anyhow::Result<Self> {
        if db_file.is_file() {
            let data = std::fs::read(&db_file)?;
            let dump: DatabaseDump = from_slice(&data)?;
            let addresses = AddressDatabase::from_addresses(dump.addresses);
            Ok(Self { db_file, addresses })
        } else {
            if db_file.parent().map(|p| p.is_dir()).unwrap_or(false) {
                std::fs::create_dir_all(db_file.parent().unwrap())?;
            } else {
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Database file parent does not exist: {:?}", db_file),
                ))?;
            }
            Ok(Self {
                db_file,
                addresses: AddressDatabase::new(),
            })
        }
    }
}

impl Drop for ProjectDatabase {
    fn drop(&mut self) {
        let dump = DatabaseDump {
            addresses: self.addresses.dump_db(),
        };
        if let Ok(data) = to_vec(&dump) {
            let _ = std::fs::write(&self.db_file, data);
        }
    }
}
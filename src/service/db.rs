#[allow(unused_imports)]
use crate::{dbg, debug, error};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use crate::{
    content::traits::Content,
    register::{ContentRegister, Id},
    service::config::config,
};

use super::editors::EditManager;

#[derive(Debug, Deserialize, Serialize)]
pub struct DBHandler {
    pub register: ContentRegister<Content, Id>,
    #[serde(skip_serializing, skip_deserializing, default = "Default::default")] // TODO: yet to be implimented properly
    pub editor: EditManager,
}

impl DBHandler {
    pub fn load() -> Result<Option<Self>> {
        let db_path = config().db_path.as_path();
        let file = match File::open(db_path) {
            Ok(file) => file,
            Err(_) => return Ok(None), // no problem is file does not exist
        };
        let mut red = BufReader::new(file);
        let mut buf = String::new();
        red.read_to_string(&mut buf)?;
        let dbh = serde_yaml::from_str(&buf)?;
        Ok(dbh)
    }

    pub fn save(&self) -> Result<()> {
        let db_path = config().db_path.as_path();
        let mut w = BufWriter::new(File::create(db_path)?);
        let yaml = serde_yaml::to_string(self)?;
        write!(w, "{yaml}")?;
        Ok(())
    }
}

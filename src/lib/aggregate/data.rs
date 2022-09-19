use crate::lib::config::Config;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Eq, PartialEq, Debug)]
pub struct Addlist {
    pub name: String,
    pub list: Vec<String>,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct AddlistSources {
    pub addlist: HashSet<String>,
    pub whitelist: Option<HashSet<String>>,
}

pub struct AddlistConfig {
    pub name: String,
    pub config: Arc<Config>,
}

impl AddlistConfig {
    pub fn new(name: &str, config: Arc<Config>) -> AddlistConfig {
        AddlistConfig {
            name: name.to_owned(),
            config,
        }
    }
    pub fn prefix(&self) -> &str {
        match &self.config.prefix {
            Some(prefix) => prefix,
            None => "",
        }
    }

    pub fn suffix(&self) -> &str {
        match &self.config.suffix {
            Some(suffix) => suffix,
            None => "",
        }
    }
}

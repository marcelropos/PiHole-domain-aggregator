use crate::data::AddlistSources;
use anyhow::{anyhow, Error};
use core::num::NonZeroUsize;
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;

const CONFIG_PATH: &str = "./data/config";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub threads: Option<NonZeroUsize>,
    pub addlist: HashMap<String, AddlistSources>,
    pub whitelist: Option<HashSet<String>>,
    pub size: Option<NonZeroUsize>,
    pub path: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

enum ConfigError {
    NotFound,
    Anyhow(Error),
}

/// Reads and parses the configuration.
pub fn parse_config() -> Result<Config, Error> {
    match parse_json() {
        Ok(config) => {
            return Ok(config);
        }
        Err(ConfigError::Anyhow(err)) => {
            return Err(err);
        }
        Err(ConfigError::NotFound) => {}
    }

    match parse_yml() {
        Ok(config) => {
            return Ok(config);
        }
        Err(ConfigError::Anyhow(err)) => {
            return Err(err);
        }
        Err(ConfigError::NotFound) => {}
    }
    // Create default config.
    let config = Config::default();
    let serialized = serde_yaml::to_string(&config)?;
    fs::write(CONFIG_PATH, serialized)?;
    Err(anyhow!("No config found! Created default config."))
}

fn parse_json() -> Result<Config, ConfigError> {
    match fs::read_to_string(format!("{CONFIG_PATH}.rs")) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(config) => Ok(config),
            Err(err) => match err.classify() {
                Category::Syntax => match serde_yaml::from_str(&raw) {
                    Ok(config) => Ok(config),
                    Err(err) => Err(ConfigError::Anyhow(err.into())),
                },
                Category::Data => Err(ConfigError::Anyhow(err.into())),
                Category::Io | Category::Eof => unreachable!(),
            },
        },
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Err(ConfigError::NotFound),
            _ => Err(ConfigError::Anyhow(err.into())),
        },
    }
}

fn parse_yml() -> Result<Config, ConfigError> {
    match fs::read_to_string(format!("{CONFIG_PATH}.yml")) {
        Ok(raw) => match serde_yaml::from_str(&raw) {
            Ok(config) => Ok(config),
            Err(err) => Err(ConfigError::Anyhow(anyhow!(err))),
        },
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Err(ConfigError::NotFound),
            _ => Err(ConfigError::Anyhow(err.into())),
        },
    }
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        let mut addlist = HashMap::with_capacity(2);

        let mut sources = HashSet::with_capacity(2);
        sources.insert("https://1.example.local".to_owned());
        sources.insert("https://2.example.local".to_owned());
        let addlist_sources = AddlistSources {
            addlist: sources,
            whitelist: Some(HashSet::from_iter(vec![
                "https://local.whitelist.local".to_owned(),
                "https://local.whitelist2.local".to_owned(),
            ])),
        };
        addlist.insert("AddlistOne".to_owned(), addlist_sources);

        let mut sources = HashSet::with_capacity(2);
        sources.insert("https://3.example.local".to_owned());
        sources.insert("https://4.example.local".to_owned());
        let addlist_sources = AddlistSources {
            addlist: sources,
            whitelist: None,
        };
        addlist.insert("AddlistTwo".to_owned(), addlist_sources);

        let mut whitelist = HashSet::with_capacity(2);
        whitelist.insert("https://global.whitelist1.local".to_owned());
        whitelist.insert("https://global.whitelist2.local".to_owned());

        // The unsafe code below never results in an error because the literals always result in valid data.
        #[allow(clippy::unwrap_used)]
        Self {
            threads: Some(NonZeroUsize::new(max(num_cpus::get() / 2, 1)).unwrap()),
            addlist,
            whitelist: Some(whitelist),
            path: "./addlists".to_owned(),
            prefix: Some("127.0.0.1 ".to_owned()),
            suffix: Some("# Some text here.".to_owned()),
            size: Some(NonZeroUsize::new(1_000_000).unwrap()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::Config;
    use std::cmp::max;

    #[test]
    fn test_config_default_threads() -> Result<(), String> {
        let config = Config::default();
        assert!(config.threads.unwrap().get() == max(num_cpus::get() / 2, 1));
        Ok(())
    }

    #[test]
    fn test_config_default_size() -> Result<(), String> {
        let config = Config::default();
        assert!(config.size.unwrap().get() == 1_000_000);
        Ok(())
    }
}

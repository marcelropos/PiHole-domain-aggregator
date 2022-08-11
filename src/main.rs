mod lib;
use crate::lib::aggregate::data::{Addlist, AddlistConfig};
use crate::lib::aggregate::lists::{addlist, whitelist};
use crate::lib::config::Config;
use crate::lib::errors::MyErrors;
use crate::lib::thread::ThreadPool;
use serde_json::error::Category;
use std::fs;
use std::io::{ErrorKind, Write};
use std::sync::Arc;

const CONFIG_PATH: &str = "./data/config";

/// After a valid configuration is parsed, the program will be started.
///
/// # Errors
/// This function will throw an error if:
/// - [MyErrors::ConfigErr] If no configuration was found, is not valid or could not parsed.
/// - [MyErrors::IoErr] with [std::io::ErrorKind] information if config could not be accessed.
fn main() -> Result<(), MyErrors> {
    let config = parse_config()?;
    run(config)
}

/// Reads and parses the configuration.
fn parse_config() -> Result<Config, MyErrors> {
    match fs::read_to_string(CONFIG_PATH) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(config) => Ok(config),
            Err(err) => match err.classify() {
                Category::Syntax => match serde_yaml::from_str(&raw) {
                    Ok(config) => Ok(config),
                    Err(err) => Err(err.into()),
                },
                Category::Data => Err(err.into()),
                Category::Io | Category::Eof => unreachable!(),
            },
        },
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                let config = Config::default();
                let serialized = serde_yaml::to_string(&config).unwrap();
                fs::write(CONFIG_PATH, serialized)?;
                Err(MyErrors::ConfigErr(
                    "No config found! Created default config.".to_owned(),
                ))
            }
            _ => Err(err.into()),
        },
    }
}

/// Creates all addlists as in the givn Config definded.
///
/// # Errors
/// - This function will return `lib::config::Config::InvalidConfig` error when the number of threads is lower than 1 or grather than a half of all logical cores.
fn run(config: Config) -> Result<(), MyErrors> {
    let pool = ThreadPool::new(config.threads)?;

    let config = Arc::new(config);
    let whitelist = Arc::new(whitelist(config.whitelist.clone(), &config).unwrap_or_default());
    for (addlist_name, _) in config.addlist.iter() {
        let addlist_config = AddlistConfig::new(addlist_name, config.clone());
        let whitelist = whitelist.clone();

        pool.execute(move || {
            if let Some(data) = addlist(&addlist_config, whitelist) {
                if let Some(err) = write_to_file(addlist_config, data).err() {
                    eprint!("{:?}", err);
                }
            }
        })
    }

    Ok(())
}

/// Writes addlist to file.
///
/// # Errors
/// This function will return the first error of non-ErrorKind::Interrupted kind that [write] returns.
fn write_to_file(config: AddlistConfig, addlist: Addlist) -> std::io::Result<()> {
    let mut file = fs::File::create(format!("{}/{}.addlist", config.config.path, addlist.name))?;
    file.write_all(addlist.list.join("\r\n").as_bytes())?;
    Ok(())
}

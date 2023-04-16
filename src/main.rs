#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::unimplemented)]
#![deny(unsafe_code)]
#![warn(clippy::filter_map_next)]
#![warn(clippy::flat_map_option)]
#![warn(clippy::implicit_clone)]

mod config;
mod thread;
mod aggregate;

use config::Config;
use thread::ThreadPool;
use aggregate::data::{Addlist, AddlistConfig};
use aggregate::lists::{addlist, whitelist};
use anyhow::{anyhow, Error};
use serde_json::error::Category;
use std::fs;
use std::io::{ErrorKind, Write};
use std::sync::Arc;

const CONFIG_PATH: &str = "./data/config";

/// After a valid configuration is parsed, the program will be started.
///
/// # Errors
/// This function will throw an error if:
/// - If no configuration was found, is not valid or could not parsed.
fn main() -> Result<(), Error> {
    let config = parse_config()?;
    run(config)
}

/// Reads and parses the configuration.
fn parse_config() -> Result<Config, Error> {
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
                let serialized = serde_yaml::to_string(&config)?;
                fs::write(CONFIG_PATH, serialized)?;
                Err(anyhow!("No config found! Created default config."))
            }
            _ => Err(err.into()),
        },
    }
}

/// Creates all addlists as in the givn Config definded.
///
/// # Errors
/// - If the Config is invalid.
fn run(config: Config) -> Result<(), Error> {
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

/// Writes addlist to (multiple) file(s).
///
/// Based on [lib::config::Config].size attribute the addlist is split into multiple files or written all at one file.
///
/// # Errors
/// - If file could not be created or manipulated.
fn write_to_file(config: AddlistConfig, addlist: Addlist) -> std::io::Result<()> {
    match config.config.size {
        Some(size) => addlist
            .list
            .chunks(size.get())
            .map(|data| data.join("\r\n"))
            .enumerate()
            .try_for_each(|(num, data)| {
                let mut file = fs::File::create(format!(
                    "{}/{}-{}.addlist",
                    config.config.path, num, addlist.name
                ))?;
                file.write_all(data.as_bytes()).map(|_| ())
            })?,
        None => {
            let mut file =
                fs::File::create(format!("{}/{}.addlist", config.config.path, addlist.name))?;
            file.write_all(addlist.list.join("\r\n").as_bytes())?
        }
    }
    Ok(())
}

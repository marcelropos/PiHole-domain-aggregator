#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::unimplemented)]
#![deny(unsafe_code)]
#![warn(clippy::filter_map_next)]
#![warn(clippy::flat_map_option)]
#![warn(clippy::implicit_clone)]

mod aggregate;
mod config;
mod data;
mod store;
mod thread;

use aggregate::lists::{addlist, whitelist};
use anyhow::Error;
use config::{parse_config, Config};
use data::AddlistConfig;
use std::sync::Arc;
use store::write_to_file;
use thread::ThreadPool;

/// After a valid configuration is parsed, the program will be started.
///
/// # Errors
/// This function will throw an error if:
/// - If no configuration was found, is not valid or could not parsed.
fn main() -> Result<(), Error> {
    run(parse_config()?)
}

/// Creates all addlists as in the givn Config definded.
///
/// # Errors
/// - If the Config is invalid.
fn run(config: Config) -> Result<(), Error> {
    let pool = ThreadPool::new(config.threads)?;

    let config = Arc::new(config);
    let whitelist = Arc::new(whitelist(config.whitelist.clone()).unwrap_or_default());
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

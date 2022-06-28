use lib::addlist::{addlist, AddlistConfig};
use lib::config::Config;
use lib::errors::MyErrors;
use lib::thread::ThreadPool;
use std::fs;
use std::io::Write;

mod lib;

const CONFIG_PATH: &str = "./data/config.yml";

/// Reads and parses the configuration.
///
/// After a valid configuration is parsed, the program will be started.
fn main() -> Result<(), MyErrors> {
    match fs::read_to_string(CONFIG_PATH) {
        Ok(config) => match serde_yaml::from_str(&config) {
            Ok(config) => run(config),
            Err(err) => Err(MyErrors::FailedToParseConfig(err.to_string())),
        },
        Err(_) => {
            let config = Config::default();
            let serialized = serde_yaml::to_string(&config).unwrap(); //This will newer fail
            if let Err(err) = fs::write(CONFIG_PATH, serialized) {
                Err(MyErrors::FailedToCreateConfig(err.to_string()))
            } else {
                Err(MyErrors::NoCofigurationFound(String::from(
                    "Created default config. Please insert your Addlists and restart",
                )))
            }
        }
    }
}

/// Creates all addlists as in the givn Config definded.
fn run(config: Config) -> Result<(), MyErrors> {
    let pool = ThreadPool::new(config.threads)?;
    for (addlist_name, _) in config.addlist.iter() {
        let addlist_config = AddlistConfig::new(addlist_name, config.clone());
        pool.execute(|| {
            let data = addlist(&addlist_config);
            write_to_file(addlist_config, data);
        })
    }

    Ok(())
}

/// Writes addlist to file.
///
/// # Panics
/// If the path to the file does not exist or no write permissions are available, the method panics.
fn write_to_file(addlist_config: AddlistConfig, mut data: Vec<String>) {
    if let Ok(mut file) = fs::File::create(format!(
        "{}/{}.addlist",
        addlist_config.config.path, addlist_config.name
    )) {
        data.sort();

        for line in data {
            file.write_all((line + "\r\n").as_bytes())
                .expect("Unable to write to file");
        }
    }
}

use super::config::Config;
use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::sync::Arc;
use std::{thread, time};

pub struct AddlistConfig {
    pub name: String,
    pub config: Arc<Config>,
    want: Arc<Regex>,
    not_want: Arc<Regex>,
}

impl AddlistConfig {
    pub fn new(
        name: &String,
        config: Arc<Config>,
        want: Arc<Regex>,
        not_want: Arc<Regex>,
    ) -> AddlistConfig {
        AddlistConfig {
            name: name.to_string(),
            config,
            want,
            not_want,
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
/// Generate Addlist
///
/// Writes generates addlist as defined in the config.
pub fn addlist(config: &AddlistConfig) -> Vec<String> {
    let client = Client::new();

    let data = config
        .config
        .as_ref()
        .addlist
        .iter()
        .filter(|list| list.0 == config.name)
        .flat_map(|list| &list.1)
        .flat_map(|url| fetch(url, &client, config.config.delay))
        .flat_map(|data| parse(data, &config))
        .collect();
    mutate(&config, data)
}

/// Fetches raw domain data
fn fetch(url: &String, client: &Client, delay: u64) -> Option<String> {
    thread::sleep(time::Duration::from_millis(delay));
    match client.get(url).send() {
        Ok(resp) => {
            if resp.status() == 200 {
                match resp.text() {
                    Ok(text) => Some(text),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Parses a raw data to a HashSet of valid domains.
///
/// Raw data is parsed to valid unique domains.
fn parse(raw_data: String, config: &AddlistConfig) -> HashSet<String> {
    raw_data
        .to_lowercase()
        .lines()
        .map(|line| match line.find("#") {
            Some(index) => line[..index].as_ref(),
            None => line,
        })
        .flat_map(|line| line.split(" "))
        .filter(|entry| !config.not_want.is_match(entry))
        .flat_map(|entry| config.want.find_iter(entry))
        .map(|domain| domain.as_str().to_string())
        .collect()
}

/// Normalises domains and adds prefix and sufix.
fn mutate(config: &AddlistConfig, domains: HashSet<String>) -> Vec<String> {
    let mut no_prefix: Vec<String> = domains
        .into_iter()
        .map(|domain| {
            if domain.split(".").count() == 3 && domain.starts_with("www.") {
                domain[4..].to_string()
            } else {
                domain.to_string()
            }
        })
        .collect();
    no_prefix.sort();

    let mut prefix: Vec<String> = no_prefix
        .iter()
        .filter(|domain| domain.split(".").count() == 2 && !domain.starts_with("www."))
        .map(|domain| format!("www.{}", domain))
        .collect();
    prefix.sort();

    no_prefix.extend(prefix);
    no_prefix
        .iter()
        .map(|domain| format!("{}{}{}", config.prefix(), domain, config.suffix()))
        .collect()
}

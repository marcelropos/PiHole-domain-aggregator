use super::super::config::Config;
use super::data::{Addlist, AddlistConfig};
use super::validation;
use core::num::NonZeroU64;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::sync::Arc;
use std::{thread, time};

pub const DOT: char = '.';
const WWW: &str = "www.";
const COMMENT: char = '#';

/// Creates Addlist
pub fn addlist(config: &AddlistConfig, global_whitelist: Arc<HashSet<String>>) -> Option<Addlist> {
    let client = Client::new();
    let sources = config.config.addlist.get(&config.name)?;
    let local_whitelist = match whitelist(sources.whitelist.clone(), &config.config) {
        Some(local_whitelist) => local_whitelist,
        None => HashSet::new(),
    };
    let local_reduced_whitelist: HashSet<_> =
        local_whitelist.difference(&global_whitelist).collect();

    let data = sources
        .addlist
        .iter()
        .flat_map(|url| fetch(url, &client, config.config.delay))
        .flat_map(parse)
        .filter(|domain| !global_whitelist.contains(domain))
        .filter(|domain| !local_reduced_whitelist.contains(domain))
        .collect();

    Some(Addlist {
        list: mutate(config, data),
        name: config.name.to_owned(),
    })
}

/// Creates Whitelist
pub fn whitelist(mut sources: Option<HashSet<String>>, config: &Config) -> Option<HashSet<String>> {
    let client = Client::new();
    let whitelist = sources
        .take()?
        .iter()
        .flat_map(|url| fetch(url, &client, config.delay))
        .flat_map(parse)
        .collect();
    Some(whitelist)
}

/// Fetches raw domain data
fn fetch(url: &String, client: &Client, delay: Option<NonZeroU64>) -> Option<String> {
    if let Some(delay) = delay {
        thread::sleep(time::Duration::from_millis(delay.get()));
    }
    let response = client.get(url).send().ok()?;
    if response.status() == 200 {
        return response.text().ok();
    }
    None
}

/// Parses a raw data to a HashSet of valid domains.
///
/// Raw data is parsed to valid unique domains.
fn parse(raw_data: String) -> HashSet<String> {
    raw_data
        .to_lowercase()
        .lines()
        .map(|line| {
            line.find(COMMENT)
                .and_then(|index| Some(line[..index].as_ref()))
                .unwrap_or(line)
        })
        .flat_map(|line| line.split_whitespace())
        .flat_map(validation::validate)
        .collect()
}

/// Muatates domains based on config.
///
/// Adds prefix and suffix as in the configuration defined.
/// Converts the Set of domains to a sorted vector.
/// Add/Remove the subdomain `www.` to have both in the addlist.
fn mutate(config: &AddlistConfig, domains: HashSet<String>) -> Vec<String> {
    let mut no_prefix = domains
        .into_iter()
        .map(|domain| {
            if domain.split(DOT).count() == 3 && domain.starts_with(WWW) {
                domain
                    .strip_prefix(WWW)
                    .unwrap_or(domain.as_str())
                    .to_owned()
            } else {
                domain
            }
        })
        .collect::<HashSet<String>>()
        .into_iter()
        .collect::<Vec<String>>();
    no_prefix.sort();

    let mut prefix = no_prefix
        .iter()
        .filter(|domain| domain.split(DOT).count() == 2 && !domain.starts_with(WWW))
        .map(|domain| format!("{}{}", WWW, domain))
        .collect::<HashSet<String>>()
        .into_iter()
        .collect::<Vec<String>>();
    prefix.sort();

    no_prefix.extend(prefix);
    no_prefix
        .into_iter()
        .map(|domain| format!("{}{}{}", config.prefix(), domain, config.suffix()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::data::{Addlist, AddlistConfig, AddlistSources};
    use super::Config;
    use mockito::mock;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    #[test]
    fn test_addlist_whitelist() -> Result<(), String> {
        // Set up environment
        let mock = mock("GET", "/addlist")
            .with_status(200)
            .with_body("docs.rs\nwww.rust-lang.org")
            .create();

        let url = &mockito::server_url();

        let mut config = Config::default();
        config.delay = None;
        config.prefix = None;
        config.suffix = None;

        let mut addlist = HashMap::new();
        addlist.insert(
            "Addlist".to_owned(),
            AddlistSources {
                addlist: HashSet::from_iter(vec![url.to_owned() + "/addlist"]),
                whitelist: None,
            },
        );
        config.addlist = addlist;

        let config = AddlistConfig {
            name: "Addlist".to_owned(),
            config: Arc::new(config),
        };

        let whitelist = Arc::new(HashSet::from_iter(vec!["www.rust-lang.org".to_owned()]));

        let have = super::addlist(&config, whitelist);
        let want = Some(Addlist {
            name: "Addlist".to_owned(),
            list: vec!["docs.rs".to_owned(), "www.docs.rs".to_owned()],
        });

        mock.assert();
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_addlist_local_whitelist() -> Result<(), String> {
        // Set up environment
        let mock1 = mock("GET", "/addlist")
            .with_status(200)
            .with_body("docs.rs\nwww.rust-lang.org\nt.org")
            .create();
        let mock2 = mock("GET", "/whitelist")
            .with_status(200)
            .with_body("docs.rs\nwww.rust-lang.org")
            .create();

        let url = &mockito::server_url();

        let mut config = Config::default();
        config.delay = None;
        config.prefix = None;
        config.suffix = None;

        let mut addlist = HashMap::new();
        addlist.insert(
            "Addlist".to_owned(),
            AddlistSources {
                addlist: HashSet::from_iter(vec![url.to_owned() + "/addlist"]),
                whitelist: Some(HashSet::from_iter(vec![url.to_owned() + "/whitelist"])),
            },
        );
        config.addlist = addlist;

        let config = AddlistConfig {
            name: "Addlist".to_owned(),
            config: Arc::new(config),
        };

        let whitelist = Arc::new(HashSet::from_iter(vec!["www.rust-lang.org".to_owned()]));

        let have = super::addlist(&config, whitelist);
        let want = Some(Addlist {
            name: "Addlist".to_owned(),
            list: vec!["t.org".to_owned(), "www.t.org".to_owned()],
        });

        mock1.assert();
        mock2.assert();
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_parse_valid() -> Result<(), String> {
        let raw = vec![
            String::from("aa") + ".ccccc".repeat(50).as_str() + ".com",
            String::from("docs.rs"),
            String::from("rust-lang.org"),
            String::from("t.org"),
            String::from("xn--mller-brombel-rmb4fg.de"),
        ];
        let want = HashSet::from_iter(raw.clone());
        let have = super::parse(raw.join("\n"));
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_parse_invaid() -> Result<(), String> {
        let raw = vec![
            String::from("-analytics/analytics."),
            String::from("::1"),
            String::from(".php?action_name="),
            String::from("/_log?ocid="),
            String::from("&action=confection_send_data&"),
            String::from("#doc.rust-lang.org"),
            String::from("||seekingalpha.com/mone_event"),
            String::from("1035.ietf.org"),
            String::from("127.0.0.1"),
            String::from("aac") + ".ccccc".repeat(50).as_str() + ".com",
            String::from(
                "rfc---------------------------------------------------------1035.ietf.org",
            ),
            String::from("rfc1035-.ietf.org"),
            String::from("rfc1035.?itf.org"),
        ];
        let want = HashSet::new();
        let have = super::parse(raw.join("\n"));
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_parse_truncate() -> Result<(), String> {
        let raw = vec![
            String::from("adserver.example.com #example.com - Advertising"),
            String::from("www.reddit.com/r/learnrust/"),
            String::from("www.rfc-editor.org."),
            String::from("www.rust-lang.org:443"),
        ];
        let want = HashSet::from_iter([
            String::from("adserver.example.com"),
            String::from("www.reddit.com"),
            String::from("www.rfc-editor.org"),
            String::from("www.rust-lang.org"),
        ]);
        let have = super::parse(raw.join("\n"));
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_parse_punicode() -> Result<(), String> {
        let raw = vec![String::from("www.müller-büromöbel.de")];
        let want = HashSet::from_iter([String::from("www.xn--mller-brombel-rmb4fg.de")]);
        let have = super::parse(raw.join("\n"));
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_mutate_add() -> Result<(), String> {
        let premut = HashSet::from_iter([
            String::from("a.com"),
            String::from("b.com"),
            String::from("c.com"),
        ]);
        let mut config = Config::default();
        config.prefix = None;
        config.suffix = None;
        let addlist_config = super::AddlistConfig {
            name: String::from("New"),
            config: Arc::new(config),
        };
        let want = vec![
            String::from("a.com"),
            String::from("b.com"),
            String::from("c.com"),
            String::from("www.a.com"),
            String::from("www.b.com"),
            String::from("www.c.com"),
        ];
        let have = super::mutate(&addlist_config, premut);
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_mutate_remove() -> Result<(), String> {
        let premut = HashSet::from_iter([
            String::from("www.a.com"),
            String::from("www.b.com"),
            String::from("www.c.com"),
        ]);
        let mut config = Config::default();
        config.prefix = None;
        config.suffix = None;
        let addlist_config = super::AddlistConfig {
            name: String::from("New"),
            config: Arc::new(config),
        };
        let want = vec![
            String::from("a.com"),
            String::from("b.com"),
            String::from("c.com"),
            String::from("www.a.com"),
            String::from("www.b.com"),
            String::from("www.c.com"),
        ];
        let have = super::mutate(&addlist_config, premut);
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_mutate_duplicate() -> Result<(), String> {
        let premut = HashSet::from_iter([
            String::from("a.com"),
            String::from("b.com"),
            String::from("c.com"),
            String::from("www.a.com"),
            String::from("www.b.com"),
            String::from("www.c.com"),
        ]);
        let mut config = Config::default();
        config.prefix = None;
        config.suffix = None;
        let addlist_config = super::AddlistConfig {
            name: String::from("New"),
            config: Arc::new(config),
        };
        let want = vec![
            String::from("a.com"),
            String::from("b.com"),
            String::from("c.com"),
            String::from("www.a.com"),
            String::from("www.b.com"),
            String::from("www.c.com"),
        ];
        let have = super::mutate(&addlist_config, premut);
        assert_eq!(want, have);
        Ok(())
    }
}

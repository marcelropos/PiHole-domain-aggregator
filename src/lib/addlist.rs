use super::config::Config;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::{thread, time};

pub struct AddlistConfig {
    pub name: String,
    pub config: Config,
}

impl AddlistConfig {
    pub fn new(name: &String, config: Config) -> AddlistConfig {
        AddlistConfig {
            name: name.to_string(),
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
/// Generate Addlist
///
/// Writes generates addlist as defined in the config.
pub fn addlist(config: &AddlistConfig) -> Vec<String> {
    let client = Client::new();

    let data = config
        .config
        .addlist
        .iter()
        .filter(|list| list.0 == config.name)
        .flat_map(|list| &list.1)
        .flat_map(|url| fetch(url, &client, config.config.delay))
        .flat_map(|data| parse(data))
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
fn parse(raw_data: String) -> HashSet<String> {
    raw_data
        .to_lowercase()
        .lines()
        .map(|line| match line.find("#") {
            Some(index) => line[..index].as_ref(),
            None => line,
        })
        .flat_map(|line| line.split(" "))
        .map(|entry| domain_validation::encode(entry))
        .map(|entry| domain_validation::truncate(entry))
        .filter(|domain| domain_validation::validate(domain))
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

mod domain_validation {

    /// Recives possible IDNs and converts it to punicode if needed.
    pub fn encode(str: &str) -> String {
        let part = str.split(".");
        let encoded: Vec<String> = part.into_iter().map(|str| help_encode(str)).collect();
        encoded.join(".")
    }

    fn help_encode(str: &str) -> String {
        if str.is_ascii() {
            str.to_string()
        } else {
            match punycode::encode(str) {
                Ok(str) => String::from("xn--") + str.as_str(),
                Err(_) => str.to_string(),
            }
        }
    }

    /// Truncates invalid characters and returns the valid part.
    pub fn truncate(str: String) -> String {
        let valid: String = str
            .chars()
            .filter(|c| !char::is_ascii_alphanumeric(&c.clone()))
            .map(|c| c.to_string())
            .filter(|c| !["-", "."].contains(&c.as_str()))
            .take(1)
            .collect();

        let result;
        if valid != "" {
            match str.find(valid.as_str()) {
                Some(index) => result = str[..index].to_string(),
                None => result = str,
            }
        } else {
            result = str
        }
        result
            .strip_suffix(".")
            .unwrap_or_else(|| result.as_str())
            .to_string()
    }

    /// Validates domain as in rfc1035 defined.
    pub fn validate(domain: &String) -> bool {
        let mut lables = domain.split(".");
        let is_first_alphabetic = lables.clone().all(|label| {
            label
                .chars()
                .nth(0)
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or_else(|| false)
        });
        let is_last_alphanumeric = lables.clone().all(|label| {
            label
                .chars()
                .last()
                .map(|c| (c.is_ascii_alphanumeric()))
                .unwrap_or_else(|| false)
        });
        let is_interior_characters_valid = lables
            .clone()
            .all(|label| label.chars().all(|c| c.is_alphanumeric() || c == '-'));
        let upper_limit = lables.clone().all(|label| label.len() <= 63);
        let lower_limit = lables.all(|label| label.len() >= 1);
        let total_upper_limit = domain.chars().filter(|c| c.to_string() != ".").count() <= 255;
        let contains_dot = domain.contains(".");

        contains_dot
            && is_first_alphabetic
            && is_last_alphanumeric
            && is_interior_characters_valid
            && upper_limit
            && lower_limit
            && total_upper_limit
    }

    mod tests {
        #[test]
        fn test_decode_no_change() -> Result<(), String> {
            assert_eq!("www.rust-lang.org", super::encode("www.rust-lang.org"));
            Ok(())
        }

        #[test]
        fn test_encode() -> Result<(), String> {
            assert_eq!(
                "www.xn--mller-brombel-rmb4fg.de",
                super::encode("www.müller-büromöbel.de")
            );
            Ok(())
        }
        #[test]
        fn test_not_truncated() -> Result<(), String> {
            assert_eq!(
                "www.rust-lang.org",
                super::truncate("www.rust-lang.org".to_string()),
                "The should not be any changes!"
            );
            Ok(())
        }

        #[test]
        fn test_truncate_port() -> Result<(), String> {
            assert_eq!(
                "www.rust-lang.org",
                super::truncate("www.rust-lang.org:443".to_string()),
                "The port was not cut off!"
            );
            Ok(())
        }

        #[test]
        fn test_truncate_uri() -> Result<(), String> {
            assert_eq!(
                "www.rust-lang.org",
                super::truncate("www.rust-lang.org/community".to_string()),
                "The request uri was not cut off!"
            );
            Ok(())
        }

        #[test]
        fn test_validate_valid() -> Result<(), String> {
            assert!(
                super::validate(&String::from("rfc-1035.ietf.org")),
                "Rejected vaid domain!"
            );

            Ok(())
        }
        #[test]
        fn test_validate_letter_or_digit() -> Result<(), String> {
            assert!(
                !super::validate(&String::from("rfc1035-.ietf.org")),
                "At least one labe does not end with a letter or a digit!"
            );
            Ok(())
        }

        #[test]
        fn test_validate_letter() -> Result<(), String> {
            assert!(
                !super::validate(&String::from("1035.ietf.org")),
                "Domain must start with a letter!"
            );
            Ok(())
        }

        #[test]
        fn test_validate_letter1() -> Result<(), String> {
            assert!(
                !super::validate(&String::from("-1035.ietf.org")),
                "Domain must start with a letter!"
            );
            Ok(())
        }
        #[test]
        fn test_validate_valid_chars() -> Result<(), String> {
            assert!(
                !super::validate(&String::from("rfc1035.i?tf.org")),
                "Domain must only contain letters digits or hivens!"
            );
            Ok(())
        }
        #[test]
        fn test_validate_short() -> Result<(), String> {
            assert!(
                !super::validate(&String::from(".org")),
                "Domains must be longer than 1 character!"
            );
            Ok(())
        }
        #[test]
        fn test_validate_long() -> Result<(), String> {
            assert!(
                !super::validate(&String::from(
                    "rfc---------------------------------------------------------1035.ietf.org"
                )),
                "Domains must be shorter than 64 character!"
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_parse_valid() -> Result<(), String> {
        let raw = vec![
            String::from("rust-lang.org"),
            String::from("docs.rs"),
            String::from("xn--mller-brombel-rmb4fg.de"),
            String::from("t.org"),
            String::from("aa") + ".ccccc".repeat(50).as_str() + ".com",
        ];
        let want = HashSet::from_iter(raw.clone());
        let have = super::parse(raw.join("\n"));
        assert_eq!(want, have);
        Ok(())
    }

    #[test]
    fn test_parse_invaid() -> Result<(), String> {
        let raw = vec![
            String::from("127.0.0.1"),
            String::from("::1"),
            String::from("#doc.rust-lang.org"),
            String::from("&action=confection_send_data&"),
            String::from("-analytics/analytics."),
            String::from(".php?action_name="),
            String::from("/_log?ocid="),
            String::from("||seekingalpha.com/mone_event"),
            String::from(
                "rfc---------------------------------------------------------1035.ietf.org",
            ),
            String::from("rfc1035.?itf.org"),
            String::from("rfc1035-.ietf.org"),
            String::from("1035.ietf.org"),
            String::from("aac") + ".ccccc".repeat(50).as_str() + ".com",
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
            String::from("www.rust-lang.org:443"),
            String::from("www.rfc-editor.org."),
        ];
        let want = HashSet::from_iter([
            String::from("adserver.example.com"),
            String::from("www.reddit.com"),
            String::from("www.rust-lang.org"),
            String::from("www.rfc-editor.org"),
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
}

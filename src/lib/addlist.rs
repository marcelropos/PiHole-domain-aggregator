use super::config::Config;
use core::num::NonZeroU64;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::sync::Arc;
use std::{thread, time};

const DOT: char = '.';

pub struct Addlist {
    pub name: String,
    pub list: Vec<String>,
}

pub struct AddlistConfig {
    pub name: String,
    pub config: Arc<Config>,
}

impl AddlistConfig {
    pub fn new(name: &String, config: Arc<Config>) -> AddlistConfig {
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
/// Creates Addlist
pub fn addlist(config: &AddlistConfig) -> Option<Addlist> {
    let client = Client::new();

    let data = config
        .config
        .addlist
        .get(&config.name)?
        .iter()
        .flat_map(|url| fetch(url, &client, config.config.delay))
        .flat_map(parse)
        .collect();

    Some(Addlist {
        list: mutate(config, data),
        name: config.name.to_owned(),
    })
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
        .map(|line| match line.find('#') {
            Some(index) => line[..index].as_ref(),
            None => line,
        })
        .flat_map(|line| line.split(' '))
        .map(domain_validation::encode)
        .map(domain_validation::truncate)
        .filter(|data| domain_validation::validate(data.as_str()))
        .collect()
}

/// Muatates domains based on config.
///
/// Adds prefix and suffix as in the configuration defined.
/// Converts the Set of domains to a sorted vector.
/// Add/Remove the subdomain `www.` to have both in the addlist.
fn mutate(config: &AddlistConfig, domains: HashSet<String>) -> Vec<String> {
    let mut no_prefix: Vec<String> = domains
        .into_iter()
        .map(|domain| {
            if domain.split(DOT).count() == 3 && domain.starts_with("www.") {
                domain
                    .strip_prefix("www.")
                    .unwrap_or(domain.as_str())
                    .to_owned()
            } else {
                domain
            }
        })
        .collect();
    no_prefix.sort();

    let mut prefix: Vec<String> = no_prefix
        .iter()
        .filter(|domain| domain.split(DOT).count() == 2 && !domain.starts_with("www."))
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
    use super::DOT;
    use std::num::NonZeroUsize;

    const HYPHEN: char = '-';
    const PUNY: &str = "xn--";
    const VALID_CHARS: [char; 2] = [HYPHEN, DOT];

    /// Recives possible IDNs and converts it to punicode if needed.
    pub fn encode(decoded: &str) -> String {
        decoded
            .split(DOT)
            .into_iter()
            .map(help_encode)
            .collect::<Vec<String>>()
            .join(".")
    }

    fn help_encode(decoded: &str) -> String {
        if decoded.is_ascii() {
            return decoded.to_owned();
        }
        punycode::encode(decoded)
            .map(|encoded| PUNY.to_owned() + encoded.as_str())
            .unwrap_or_else(|_| decoded.to_owned())
    }

    /// Truncates invalid characters and returns the valid part.
    pub fn truncate(raw: String) -> String {
        let invalid: String = raw
            .chars()
            .filter(|character| !character.is_ascii_alphanumeric())
            .filter(|character| !VALID_CHARS.contains(character))
            .take(1)
            .collect();

        let raw = raw
            .find(invalid.as_str())
            .map(NonZeroUsize::new)
            .and_then(|index| index)
            .map(|index| raw[..index.get()].to_owned())
            .unwrap_or(raw);

        raw.strip_suffix(DOT)
            .map(|truncated| truncated.to_owned())
            .unwrap_or(raw)
    }

    /// Validates domain as in rfc1035 defined.
    pub fn validate(domain: &str) -> bool {
        let mut lables = domain.split(DOT);
        let is_first_alphabetic = lables.clone().all(|label| {
            label
                .chars()
                .next()
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
            .all(|label| label.chars().all(|c| c.is_alphanumeric() || HYPHEN.eq(&c)));
        let upper_limit = lables.clone().all(|label| label.len() <= 63);
        let lower_limit = lables.all(|label| !label.is_empty());
        let total_upper_limit = domain.chars().filter(|c| !DOT.eq(c)).count() <= 255;
        let contains_dot = domain.contains(DOT);

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
                super::truncate("www.rust-lang.org".to_owned()),
                "The should not be any changes!"
            );
            Ok(())
        }

        #[test]
        fn test_truncate_port() -> Result<(), String> {
            assert_eq!(
                "www.rust-lang.org",
                super::truncate("www.rust-lang.org:443".to_owned()),
                "The port was not cut off!"
            );
            Ok(())
        }

        #[test]
        fn test_truncate_uri() -> Result<(), String> {
            assert_eq!(
                "www.rust-lang.org",
                super::truncate("www.rust-lang.org/community".to_owned()),
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
    use super::Config;
    use std::collections::HashSet;
    use std::sync::Arc;

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
    fn test_mutate() -> Result<(), String> {
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
}

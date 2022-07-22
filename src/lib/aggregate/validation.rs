use super::lists::DOT;
use std::num::NonZeroUsize;

const HYPHEN: char = '-';
const PUNY: &str = "xn--";
const VALID_CHARS: [char; 2] = [HYPHEN, DOT];

/// Validates domain as in rfc1035 defined.
pub fn validate(domain: &str) -> Option<String> {
    let domain = truncate(encode(domain));
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

    if contains_dot
        && is_first_alphabetic
        && is_last_alphanumeric
        && is_interior_characters_valid
        && upper_limit
        && lower_limit
        && total_upper_limit
    {
        Some(domain)
    } else {
        None
    }
}

/// Recives possible IDNs and converts it to punicode if needed.
fn encode(decoded: &str) -> String {
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
fn truncate(raw: String) -> String {
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
        assert_eq!(
            super::validate("rfc-1035.ietf.org"),
            Some("rfc-1035.ietf.org".to_owned()),
            "Rejected vaid domain!"
        );

        Ok(())
    }
    #[test]
    fn test_validate_letter_or_digit() -> Result<(), String> {
        assert_eq!(
            super::validate("rfc1035-.ietf.org"),
            None,
            "At least one label does not end with a letter or a digit!"
        );
        Ok(())
    }

    #[test]
    fn test_validate_letter() -> Result<(), String> {
        assert_eq!(
            super::validate("1035.ietf.org"),
            None,
            "Domain must start with a letter!"
        );
        Ok(())
    }

    #[test]
    fn test_validate_letter1() -> Result<(), String> {
        assert_eq!(
            super::validate("-1035.ietf.org"),
            None,
            "Domain must start with a letter!"
        );
        Ok(())
    }
    #[test]
    fn test_validate_valid_chars() -> Result<(), String> {
        assert_eq!(
            super::validate("rfc1035.?itf.org"),
            None,
            "Domain must only contain letters digits or hivens!"
        );
        Ok(())
    }
    #[test]
    fn test_validate_short() -> Result<(), String> {
        assert_eq!(
            super::validate(".org"),
            None,
            "Domains must be longer than 1 character!"
        );
        Ok(())
    }
    #[test]
    fn test_validate_long() -> Result<(), String> {
        assert_eq!(
            super::validate(
                "rfc---------------------------------------------------------1035.ietf.org"
            ),
            None,
            "Domains must be shorter than 64 character!"
        );
        Ok(())
    }
}

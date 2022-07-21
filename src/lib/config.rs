use core::num::{NonZeroU64, NonZeroUsize};
use num_cpus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub version: u8,
    pub threads: NonZeroUsize,
    pub addlist: HashMap<String, Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    pub path: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub delay: Option<NonZeroU64>,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        let mut addlist = HashMap::new();
        addlist.insert(
            "AddlistOne".to_owned(),
            vec![
                "https://1.example.local".to_owned(),
                "https://2.example.local".to_owned(),
            ],
        );
        addlist.insert(
            "AddlistTwo".to_owned(),
            vec![
                "https://3.example.local".to_owned(),
                "https://4.example.local".to_owned(),
            ],
        );

        Self {
            version: 1,
            threads: NonZeroUsize::new(num_cpus::get() / 2)
                .unwrap_or_else(|| NonZeroUsize::new(1).unwrap()),
            addlist,
            whitelist: Some(vec![
                "https://whitelist1.example.local".to_owned(),
                "https://whitelist2.example.local".to_owned(),
            ]),
            path: "./".to_owned(),
            prefix: Some("127.0.0.1 ".to_owned()),
            suffix: Some("# Some text here.".to_owned()),
            delay: Some(NonZeroU64::new(1000).unwrap()),
        }
    }
}

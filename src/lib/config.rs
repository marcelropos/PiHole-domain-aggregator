use crate::lib::aggregate::data::AddlistSources;
use core::num::{NonZeroU64, NonZeroUsize};
use num_cpus;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub version: u8,
    pub threads: NonZeroUsize,
    pub addlist: HashMap<String, AddlistSources>,
    pub whitelist: Option<HashSet<String>>,
    pub size: Option<NonZeroUsize>,
    pub path: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub delay: Option<NonZeroU64>,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        let mut addlist = HashMap::with_capacity(2);

        let mut sources = HashSet::with_capacity(2);
        sources.insert("https://1.example.local".to_owned());
        sources.insert("https://2.example.local".to_owned());
        let addlist_sources = AddlistSources {
            addlist: sources,
            whitelist: Some(HashSet::from_iter(vec![
                "https://local.whitelist.local".to_owned(),
                "https://local.whitelist2.local".to_owned(),
            ])),
        };
        addlist.insert("AddlistOne".to_owned(), addlist_sources);

        let mut sources = HashSet::with_capacity(2);
        sources.insert("https://3.example.local".to_owned());
        sources.insert("https://4.example.local".to_owned());
        let addlist_sources = AddlistSources {
            addlist: sources,
            whitelist: None,
        };
        addlist.insert("AddlistTwo".to_owned(), addlist_sources);

        let mut whitelist = HashSet::with_capacity(2);
        whitelist.insert("https://global.whitelist1.local".to_owned());
        whitelist.insert("https://global.whitelist2.local".to_owned());

        // The unsafe code below never results in an error because the literals always result in valid data.
        Self {
            version: 1,
            threads: NonZeroUsize::new(num_cpus::get() / 2)
                .unwrap_or_else(|| unsafe { NonZeroUsize::new(1).unwrap_unchecked() }),
            addlist,
            whitelist: Some(whitelist),
            path: "./".to_owned(),
            prefix: Some("127.0.0.1 ".to_owned()),
            suffix: Some("# Some text here.".to_owned()),
            delay: Some(unsafe { NonZeroU64::new(1_000).unwrap_unchecked() }),
            size: Some(unsafe { NonZeroUsize::new(1_000_000).unwrap_unchecked() }),
        }
    }
}

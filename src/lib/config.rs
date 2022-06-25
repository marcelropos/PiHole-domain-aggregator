use num_cpus;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub version: u8,
    pub threads: usize,
    pub addlist: Vec<(String, Vec<String>)>,
    pub whitelist: Option<Vec<String>>,
    pub path: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub delay: u64,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        let addlist = vec![
            (
                "AddlistOne".to_string(),
                vec![
                    "https://1.example.local".to_string(),
                    "https://2.example.local".to_string(),
                ],
            ),
            (
                "AddlistTwo".to_string(),
                vec![
                    "https://3.example.local".to_string(),
                    "https://4.example.local".to_string(),
                ],
            ),
        ];

        Self {
            version: 1,
            threads: num_cpus::get() / 2,
            addlist: addlist,
            whitelist: Some(vec![
                "https://whitelist1.example.local".to_string(),
                "https://whitelist2.example.local".to_string(),
            ]),
            path: "./".to_string(),
            prefix: Some("127.0.0.1 ".to_string()),
            suffix: Some("# Some text here.".to_string()),
            delay: 1000,
        }
    }
}

use serde_yaml;
use std::convert::Into;
use std::io;

#[derive(Debug)]
pub enum MyErrors {
    FailedToParseConfig(String),
    InvalidConfig(String),
    FailedToCreateConfig(String),
    NoCofigurationFound(String),
    FailedToCreateAddlist(String),
}

impl Into<MyErrors> for io::Error {
    fn into(self) -> MyErrors {
        MyErrors::FailedToCreateConfig(self.to_string())
    }
}
impl Into<MyErrors> for serde_yaml::Error {
    fn into(self) -> MyErrors {
        MyErrors::FailedToParseConfig(self.to_string())
    }
}

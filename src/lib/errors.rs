use serde_json;
use serde_yaml;

#[derive(Debug)]
pub enum MyErrors {
    ConfigErr(String),
    IoErr(String),
}

impl From<std::io::Error> for MyErrors {
    fn from(err: std::io::Error) -> MyErrors {
        MyErrors::IoErr(err.to_string())
    }
}

impl From<serde_yaml::Error> for MyErrors {
    fn from(err: serde_yaml::Error) -> MyErrors {
        MyErrors::ConfigErr(err.to_string())
    }
}

impl From<serde_json::Error> for MyErrors {
    fn from(err: serde_json::Error) -> MyErrors {
        MyErrors::ConfigErr(err.to_string())
    }
}

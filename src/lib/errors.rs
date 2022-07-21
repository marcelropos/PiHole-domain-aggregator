use serde_yaml;

#[derive(Debug)]
pub enum MyErrors {
    ConfigErr(String),
    InvalidConfig(String),
    IoErr(String),
    NoCofigurationFound(String),
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

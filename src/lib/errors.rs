#[derive(Debug)]
pub enum MyErrors {
    FailedToParseConfig(String),
    InvalidConfig(String),
    FailedToCreateConfig(String),
    NoCofigurationFound(String),
}

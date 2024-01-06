#[derive(Debug, serde::Serialize, Clone)]
pub struct Error {
    pub message: String,
}

impl From<Error> for serde_json::Value {
    fn from(error: Error) -> Self {
        serde_json::to_value(error).unwrap()
    }
}

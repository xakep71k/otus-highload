#[derive(Debug, serde::Serialize, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_value(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

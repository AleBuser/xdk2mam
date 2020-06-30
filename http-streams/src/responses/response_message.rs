use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMessage {
    pub status: &'static str,
    pub message: String,
}

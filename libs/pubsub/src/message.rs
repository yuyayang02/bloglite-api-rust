use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct Message {
    id: String,
    payload: Vec<u8>,
    timestamp: DateTime<Local>,
}

impl<T: Serialize> From<T> for Message {
    fn from(value: T) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            payload: serde_json::to_string(&value).unwrap().into_bytes(),
            timestamp: Local::now(),
        }
    }
}

impl Message {
    pub fn payload_as<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        serde_json::from_slice::<T>(&self.payload).ok()
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    pub fn time(&self) -> DateTime<Local> {
        self.timestamp
    }
}

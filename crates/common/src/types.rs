use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub instance_id: Uuid,
}

impl ServiceInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            instance_id: Uuid::new_v4(),
        }
    }
}

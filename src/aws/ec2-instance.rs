use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct EC2Instance {
    pub id: String,
    state: Option<String>,
    name: Option<String>,
}

impl EC2Instance {
    pub fn new(new_instance_id: &str) -> Self {
        Self {
            id: String::from(new_instance_id),
            name: None,
            state: None,
        }
    }
    pub fn get_name(&self) -> String {
        match &self.name {
            Some(name) => String::from(name),
            None => String::new(),
        }
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(String::from(name));
    }
    pub fn get_state(&self) -> String {
        match &self.state {
            Some(name) => String::from(name),
            None => "(Unknown)".to_string(),
        }
    }
    pub fn set_state(&mut self, state: &str) {
        self.state = Some(String::from(state));
    }
}

impl fmt::Display for EC2Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.id, self.get_name(),)
    }
}

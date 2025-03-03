use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct EC2Instance {
    pub id: String,
    state: Option<String>,
    name: Option<String>,
    autoscaling_group_name: Option<String>,
    health: Option<String>,
}

impl EC2Instance {
    pub(crate) fn clone(&self) -> EC2Instance {
        EC2Instance {
            id: self.id.clone(),
            state: self.state.clone(),
            name: self.name.clone(),
            autoscaling_group_name: self.autoscaling_group_name.clone(),
            health: self.health.clone(),
        }
    }
}

impl EC2Instance {
    pub fn new(new_instance_id: &str) -> Self {
        Self {
            id: String::from(new_instance_id),
            name: None,
            state: None,
            autoscaling_group_name: None,
            health: None,
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

    pub fn get_autoscaling_group_name(&self) -> String {
        match &self.autoscaling_group_name {
            Some(name) => String::from(name),
            None => "(Unknown)".to_string(),
        }
    }

    pub fn set_autoscaling_group_name(&mut self, name: &str) {
        self.autoscaling_group_name = Some(String::from(name));
    }

    pub fn get_health(&self) -> String {
        match &self.health {
            Some(health) => String::from(health),
            None => "(Unknown)".to_string(),
        }
    }
    pub fn set_health(&mut self, health: &str) {
        self.health = Some(String::from(health));
    }
}

impl fmt::Display for EC2Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {} {}", self.id, self.get_health(), self.get_name(),)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec2_instance_new() {
        let instance_id = "i-1234567890abcdef";
        let instance = EC2Instance::new(instance_id);

        assert_eq!(instance.id, instance_id);
        assert_eq!(instance.name, None);
        assert_eq!(instance.state, None);
        assert_eq!(instance.autoscaling_group_name, None);
        assert_eq!(instance.health, None);
    }

    #[test]
    fn test_set_and_get_name() {
        let mut instance = EC2Instance::new("i-1234567890abcdef");
        instance.set_name("TestInstance");

        assert_eq!(instance.get_name(), "TestInstance");
        assert_eq!(instance.name, Some("TestInstance".to_string()));
    }

    #[test]
    fn test_set_and_get_state() {
        let mut instance = EC2Instance::new("i-1234567890abcdef");
        instance.set_state("running");

        assert_eq!(instance.get_state(), "running");
        assert_eq!(instance.state, Some("running".to_string()));
    }

    #[test]
    fn test_set_and_get_autoscaling_group_name() {
        let mut instance = EC2Instance::new("i-1234567890abcdef");
        instance.set_autoscaling_group_name("TestAutoScalingGroup");

        assert_eq!(
            instance.get_autoscaling_group_name(),
            "TestAutoScalingGroup"
        );
        assert_eq!(
            instance.autoscaling_group_name,
            Some("TestAutoScalingGroup".to_string())
        );
    }

    #[test]
    fn test_set_and_get_health() {
        let mut instance = EC2Instance::new("i-1234567890abcdef");
        instance.set_health("Healthy");

        assert_eq!(instance.get_health(), "Healthy");
        assert_eq!(instance.health, Some("Healthy".to_string()));
    }

    #[test]
    fn test_display() {
        let mut instance = EC2Instance::new("i-1234567890abcdef");
        instance.set_name("TestInstance");
        instance.set_health("Healthy");

        let display_output = format!("{}", instance);
        assert!(display_output.contains("i-1234567890abcdef"));
        assert!(display_output.contains("Healthy"));
        assert!(display_output.contains("TestInstance"));
    }
}

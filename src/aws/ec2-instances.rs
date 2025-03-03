use crate::aws::ec2_instance::EC2Instance;
use aws_config::SdkConfig;
use aws_sdk_ec2::types::Filter;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct EC2InstanceCollection {
    region: Option<String>,
    profile: Option<String>,
    instances: Vec<EC2Instance>,
}

impl fmt::Display for EC2InstanceCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        for instance in self.instances.iter() {
            output.push_str(format!("{}\n", instance,).as_str());
        }
        write!(f, "Instances: \n{}", output)
    }
}

impl EC2InstanceCollection {
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
    fn add_instance(&mut self, instance: EC2Instance) {
        self.instances.push(instance);
    }
    pub fn iter(&self) -> impl Iterator<Item = &EC2Instance> + '_ {
        self.instances.iter()
    }
    pub fn new() -> Self {
        Self {
            instances: vec![],
            region: None,
            profile: None,
        }
    }
    pub async fn load(config: &SdkConfig) -> Self {
        let mut instances = EC2InstanceCollection::new();
        instances.region = match config.region() {
            Some(r) => Some(format!("{}", r)),
            None => None,
        };
        Self::load_instances(&config, &mut instances).await;
        Self::load_instance_health(&config, &mut instances).await;
        instances
    }

    async fn load_instances(config: &SdkConfig, instances: &mut EC2InstanceCollection) {
        let client = aws_sdk_ec2::Client::new(&config);
        let describe_instances_result = client
            .describe_instances()
            .filters(
                Filter::builder()
                    .name("instance-state-name")
                    .values("running")
                    .build(),
            )
            .send()
            .await
            .unwrap();

        for reservation in describe_instances_result.reservations() {
            for instance in reservation.instances() {
                let instance_id = instance.instance_id().unwrap();
                let mut ec2_instance = EC2Instance::new(instance_id);

                let mut name: Option<&str> = None;
                let mut autoscaling_group_name: Option<&str> = None;
                for tag in instance.tags() {
                    match tag.key().unwrap_or_default() {
                        "Name" => name = Some(tag.value().unwrap_or_default()),
                        "aws:autoscaling:groupName" => {
                            autoscaling_group_name = Some(tag.value().unwrap_or_default())
                        }
                        _ => (),
                    }
                }
                ec2_instance.set_name(name.unwrap_or_default());
                ec2_instance.set_autoscaling_group_name(autoscaling_group_name.unwrap_or_default());

                let state: Option<&str> = match instance.state().unwrap().name() {
                    Some(state_new) => Some(state_new.as_str()),
                    None => None,
                };
                ec2_instance.set_state(state.unwrap_or_default());

                instances.add_instance(ec2_instance);
            }
        }
    }

    async fn load_instance_health(config: &SdkConfig, instances: &mut EC2InstanceCollection) {
        let mut autoscaling_group_names = HashSet::new();
        for instance in instances.instances.iter() {
            autoscaling_group_names.insert(instance.get_autoscaling_group_name());
        }
        println!("autoscaling_group_names: {:?}", autoscaling_group_names);
        let client = aws_sdk_autoscaling::Client::new(&config);
        let describe_asg_result = client
            .describe_auto_scaling_groups()
            .set_auto_scaling_group_names(Some(
                autoscaling_group_names
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ))
            .send()
            .await
            .unwrap();
        let mut instance_healths = HashMap::new();
        for group in describe_asg_result.auto_scaling_groups() {
            for instance in group.instances() {
                let instance_id = instance.instance_id().unwrap();
                let health_status = instance.health_status().unwrap();
                instance_healths.insert(instance_id, health_status);
            }
        }
        println!("instance_healths: {:?}", instance_healths);
        for instance in instances.instances.iter_mut() {
            let instance_id = instance.id.as_str();
            if instance_healths.contains_key(&instance_id) {
                let instance_health = instance_healths[&instance_id];
                instance.set_health(instance_health);
            }
        }
    }
}

extern crate itertools;
use crate::aws::ec2_instance::EC2Instance;
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_ec2::types::Filter;
use aws_sdk_ec2::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;

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
    pub fn get_region(&self) -> String {
        match &self.region {
            Some(region) => format!(" --region {}", region),
            None => String::new(),
        }
    }
    pub fn get_profile(&self) -> String {
        match &self.profile {
            Some(profile) => format!(" --profile {}", profile),
            None => String::new(),
        }
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
    async fn setup_sdk(region: &Option<String>, profile: &Option<String>) -> SdkConfig {
        let region_provider = RegionProviderChain::first_try(region.clone().map(Region::new))
            .or_default_provider()
            .or_else(Region::new("eu-west-1"));
        match &profile {
            Some(profile_string) => {
                aws_config::defaults(BehaviorVersion::latest())
                    .profile_name(profile_string)
                    .load()
                    .await
            }
            None => {
                aws_config::defaults(BehaviorVersion::latest())
                    .region(region_provider)
                    .load()
                    .await
            }
        }
    }
    pub async fn load_instances(region: &Option<String>, profile: &Option<String>) -> Self {
        let sdk_config = Self::setup_sdk(region, profile).await;
        let ec2_client = Client::new(&sdk_config);

        let mut instances = EC2InstanceCollection::new();
        instances.profile = profile.clone();
        instances.region = match sdk_config.region() {
            Some(r) => Some(format!("{}", r)),
            None => None,
        };

        let describe_instances_result = ec2_client
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

        let mut autoscaling_group_names = HashSet::new();

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

                if autoscaling_group_name.is_some() {
                    autoscaling_group_names.insert(autoscaling_group_name.unwrap());
                }

                let state: Option<&str> = match instance.state().unwrap().name() {
                    Some(state_new) => Some(state_new.as_str()),
                    None => None,
                };
                ec2_instance.set_state(state.unwrap_or_default());

                instances.add_instance(ec2_instance);
            }
        }

        let autoscaling_client = aws_sdk_autoscaling::Client::new(&sdk_config);

        let v: Vec<String> = autoscaling_group_names.iter().map(|s|s.to_string()).collect();
        let describe_asg_result = autoscaling_client
            .describe_auto_scaling_groups()
            .set_auto_scaling_group_names(Some(v))
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
        for instance in instances.instances.iter_mut() {
            let instance_id = instance.id.as_str();
            if instance_healths.contains_key(&instance_id) {
                let instance_health = instance_healths[&instance_id];
                instance.set_health(instance_health);
            }
        }

        // Self::describe_auto_scaling_groups(&sdk_config, &autoscaling_groups, &instances).await;

        instances
    }
}

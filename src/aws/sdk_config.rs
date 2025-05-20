use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region, SdkConfig};

pub async fn setup_sdk(region: &Option<String>, profile: &Option<String>) -> SdkConfig {
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

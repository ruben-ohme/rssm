use clap::Parser;

mod aws;
mod ui;
use aws::ec2_instances::EC2InstanceCollection;
use ui::run_ui;

use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region, SdkConfig};

#[derive(Debug, Parser)]
struct Opt {
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(short, long)]
    profile: Option<String>,
}


#[tokio::main]
async fn main() -> Result<(), ()> {
    let Opt { region, profile } = Opt::parse();
    let sdk_config = setup_sdk(&region, &profile).await;
    // TODO: handle SSO auth here
    let instances = EC2InstanceCollection::load(&sdk_config).await;

    run_ui(instances, region.clone(), profile.clone());
    Ok(())
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

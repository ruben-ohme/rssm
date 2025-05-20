use clap::Parser;

mod aws;
mod ui;
use aws::ec2_instances::EC2InstanceCollection;
use aws::sdk_config::setup_sdk;
use ui::run_ui;

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

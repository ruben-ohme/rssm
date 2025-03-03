use arboard::Clipboard;
use clap::Parser;

mod aws;
use aws::ec2_instances::EC2InstanceCollection;

use cursive::traits::*;
use cursive::views::{Dialog, Panel, SelectView};
use cursive::Cursive;

use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region, SdkConfig};

#[derive(Debug, Parser)]
struct Opt {
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(short, long)]
    profile: Option<String>,
}

struct SessionManagerParams {
    region: Option<String>,
    profile: Option<String>,
    target: String,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let Opt { region, profile } = Opt::parse();
    let sdk_config = setup_sdk(&region, &profile).await;
    let instances = EC2InstanceCollection::load(&sdk_config).await;

    let mut cursive = cursive::default();
    cursive.load_toml(include_str!("style.toml")).unwrap();
    cursive.add_global_callback('q', |c| c.quit());

    let region_param = match &region {
        Some(region_string) => region_string.to_string(),
        None => String::new(),
    };
    let profile_param = match &profile {
        Some(profile_string) => profile_string.to_string(),
        None => String::new(),
    };

    if !instances.is_empty() {
        let mut instance_select = SelectView::new();
        for instance in instances.iter() {
            instance_select.add_item(
                format!("{}", &instance),
                format!(
                    "{} {} {}",
                    &instance.get_name(),
                    region_param,
                    profile_param
                ),
            );
        }
        instance_select.set_on_submit(show_cmd_dialog);
        cursive.add_layer(
            Panel::new(instance_select)
                .title("Select an instance")
                .full_screen(),
        );
    } else {
        cursive.add_layer(
            Dialog::text(
                "No Instances were returned.\nCheck your region and AWS Credentials settings."
                    .to_string(),
            )
            .title("Error")
            .button("Close", |c| c.quit()),
        )
    }
    cursive.run();
    Ok(())
}

fn spawn_ssm_process(msg_input: &'static str) {
    let mut binding = std::process::Command::new("aws");
    let cmd = binding.arg("ssm").arg("start-session").arg(msg_input);
    cmd.spawn().unwrap();
    std::process::exit(0);
}

fn show_cmd_dialog(cursive: &mut Cursive, msg_input: &str) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(msg_input).unwrap();
    cursive.add_layer(
        Dialog::text(format!("{}\n Copied to clipboard.", msg_input))
            .title("Connect")
            .button("Close", |c| c.quit()) //Default!
            .button("Back", |c| close_page(c)),
    )
}

fn close_page(cursive: &mut Cursive) {
    cursive.pop_layer();
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

use arboard::Clipboard;
use clap::Parser;
use std::process::Command;

mod aws;
use aws::ec2_instance::EC2Instance;
use aws::ec2_instances::EC2InstanceCollection;

use cursive::traits::*;
use cursive::views::{Dialog, Panel, SelectView};
use cursive::{Cursive, CursiveRunnable};

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
    // TODO: handle SSO auth here
    let instances = EC2InstanceCollection::load(&sdk_config).await;

    let mut cursive = cursive::default();
    cursive.load_toml(include_str!("style.toml")).unwrap();
    cursive.add_global_callback('q', |c| c.quit());

    if instances.is_empty() {
        let logged_in = sso_login(&mut cursive, &profile);
        if !logged_in {
            cursive.add_layer(
                Dialog::text(
                    "No Instances were returned.\nCheck your region and AWS Credentials settings."
                        .to_string(),
                )
                .title("Error")
                .button("Quit", |c| c.quit()),
            );
        }
    }
    if !instances.is_empty() {
        let mut instance_select_view = SelectView::new();
        let mut sorted_instances: Vec<EC2Instance> = Vec::new();
        for instance in instances.iter() {
            sorted_instances.push(instance.clone());
        }
        sorted_instances.sort_by_key(|i| i.id.clone());
        sorted_instances.sort_by_key(|i| i.get_name());
        for instance in sorted_instances.iter() {
            instance_select_view.add_item(
                format!("{}", &instance),
                SessionManagerParams {
                    region: region.clone(),
                    profile: profile.clone(),
                    target: instance.id.clone(),
                },
            );
        }
        instance_select_view.set_on_submit(show_cmd_dialog);
        cursive.add_layer(
            Panel::new(instance_select_view)
                .title("Select an instance")
                .full_screen(),
        );
    }
    cursive.run();
    Ok(())
}

fn sso_login(cursive: &mut CursiveRunnable, profile: &Option<String>) -> bool {
    let p = profile.clone().unwrap_or_default();
    cursive
        .add_layer(Dialog::text(format!("Logging in using profile {}", p)).title("AWS SSO Login"));
    let cmd = Command::new("aws")
        .args(["sso", "login", "--profile", p.as_str()])
        .spawn();
    match cmd {
        Ok(mut child) => {
            close_page(cursive);
            child.wait().unwrap();
            true
        }
        Err(e) => {
            close_page(cursive);
            cursive.add_layer(
                Dialog::text(format!("Error: {}", e))
                    .title("Error")
                    .button("Back", |c| close_page(c)),
            );
            false
        }
    }
}

fn spawn_ssm_process(cursive: &mut Cursive, params: &SessionManagerParams) {
    show_cmd_dialog(cursive, params);
    let cmd = Command::new("aws")
        .args([
            "ssm",
            "start-session",
            "--profile",
            &params.profile.clone().unwrap_or_default(),
            "--target",
            &params.target,
        ])
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    // std::process::exit(0);
}

fn show_cmd_dialog(cursive: &mut Cursive, params: &SessionManagerParams) {
    let mut clipboard = Clipboard::new().unwrap();
    let str = format!(
        "aws ssm start-session --target {} --profile {}",
        params.target,
        params.profile.clone().unwrap_or_default()
    );
    clipboard.set_text(str.clone()).unwrap();
    cursive.add_layer(
        Dialog::text(str)
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

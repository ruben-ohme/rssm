use std::process::Command;

use arboard::Clipboard;
use cursive::traits::*;
use cursive::views::{Dialog, Panel, SelectView};
use cursive::{Cursive, CursiveRunnable};

use crate::aws::ec2_instance::EC2Instance;
use crate::aws::ec2_instances::EC2InstanceCollection;

pub struct SessionManagerParams {
    pub region: Option<String>,
    pub profile: Option<String>,
    pub target: String,
}

pub fn run_ui(
    instances: EC2InstanceCollection,
    region: Option<String>,
    profile: Option<String>,
) {
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
        sorted_instances.sort_by_key(|i| i.get_health());
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
}

fn sso_login(cursive: &mut CursiveRunnable, profile: &Option<String>) -> bool {
    let p = profile.clone().unwrap_or_default();
    cursive.add_layer(Dialog::text(format!("Logging in using profile {}", p)).title("AWS SSO Login"));
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

#[allow(dead_code)]
fn spawn_ssm_process(cursive: &mut Cursive, params: &SessionManagerParams) {
    show_cmd_dialog(cursive, params);
    let _cmd = Command::new("aws")
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
            .button("Close", |c| c.quit())
            .button("Back", |c| close_page(c)),
    )
}

fn close_page(cursive: &mut Cursive) {
    cursive.pop_layer();
}


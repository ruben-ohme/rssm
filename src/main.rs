use clap::Parser;

mod aws;
use aws::ec2_instances::EC2GetMetadata;
use aws::ec2_instances::EC2InstanceCollection;

use cursive::traits::*;
use cursive::views::{Dialog, Panel, SelectView};
use cursive::Cursive;

use arboard::Clipboard;

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
    let instances =
        EC2InstanceCollection::new_from_region(region, profile, EC2GetMetadata(true)).await;

    let mut cursive_runnable = cursive::default();
    cursive_runnable.load_toml(include_str!("style.toml")).unwrap();
    cursive_runnable.add_global_callback('q', |s| s.quit());

    if !(instances.is_empty()) {
        let mut instance_select = SelectView::new();
        instance_select.set_on_submit(show_cmd_dialog);
        for instance in instances.iter() {
            instance_select.add_item(
                format!("{}", &instance),
                format!(
                    "aws ssm start-session --target {}{}{}",
                    &instance.id,
                    &instances.get_profile(),
                    &instances.get_region(),
                ),
            );
        }
        instance_select.sort_by(|a, b| a.cmp(b));
        cursive_runnable.add_layer(
            Panel::new(instance_select)
                .title("Select an instance")
                .full_screen(),
        );
    } else {
        let no_instances_error_string =
            "No Instances were returned.\nCheck your region and AWS Credentials settings."
                .to_string();
        cursive_runnable.add_layer(
            Dialog::text(no_instances_error_string)
                .title("Error")
                .button("Close", |s| s.quit()), //Default!
        )
    }
    cursive_runnable.run();
    Ok(())
}

fn show_cmd_dialog(cursive: &mut Cursive, msg_input: &str) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(msg_input).unwrap();
    cursive.add_layer(
        Dialog::text(format!("{}\n Copied to clipboard.", msg_input))
            .title("Command")
            .button("Close", |c| c.quit()) //Default!
            .button("Back", |c| close_page(c)),
    )
}

fn close_page(cursive: &mut Cursive) {
    cursive.pop_layer();
}

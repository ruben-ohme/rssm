use clap::Parser;

mod aws;
use aws::ec2_instances::EC2InstanceCollection;

use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, Panel, SelectView};

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
    let instances = EC2InstanceCollection::load_instances(&region, &profile).await;

    let mut cursive = cursive::default();
    cursive.load_toml(include_str!("style.toml")).unwrap();
    cursive.add_global_callback('q', |c| c.quit());

    if !instances.is_empty() {
        let mut instance_select = SelectView::new();
        for instance in instances.iter() {
            instance_select.add_item(
                format!("{}", &instance),
                format!(
                    "aws ssm start-session --target {}{}{}",
                    &instance.id,
                    &instances.get_region(),
                    &instances.get_profile(),
                ),
            );
        }
        instance_select.sort_by(|a, b| a.cmp(b));
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
            .button("Close", |c| c.quit()), //Default!
        )
    }
    cursive.run();
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

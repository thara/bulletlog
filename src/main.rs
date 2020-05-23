use chrono::prelude::*;
use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1", author = "Tomochika Hara <bulletlog@thara.dev>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "add", alias = "a", about = "Add an entry")]
    Add {
        note: String,
        #[clap(short = "d")]
        date: Option<String>,
    },
}

fn today() -> String {
    let today = Local::today();
    Date::format(&today, "%Y-%m-%d").to_string()
}

fn main() {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Add { note, date } => {
            let date = date.unwrap_or_else(|| today());
            println!("{} {}", note, date);
        }
    }
}

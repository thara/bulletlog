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
    Add { note: String },
}

fn main() {
    let opts = Opts::parse();
    println!("Hello, world!");
}

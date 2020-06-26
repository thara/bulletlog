use bulletlog;

use std::error::Error;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1", author = "Tomochika Hara <bulletlog@thara.dev>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "add", aliases = &["a", "note", "n"], about = "Add an note")]
    Note { note: String },
    #[clap(name = "task", alias = "t", about = "Add an task")]
    Task { note: String },

    #[clap(name = "notes", alias = "ns", about = "List all notes")]
    ListNotes {},
    #[clap(name = "tasks", alias = "ts", about = "List all tasks")]
    ListTasks {},

    #[clap(name = "comp", alias = "c", about = "Complete a task")]
    CompleteTask { task_number: u64 },
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Note { note } => bulletlog::add_note(&note)?,
        SubCommand::Task { note } => bulletlog::add_task(&note)?,
        SubCommand::ListNotes {} => bulletlog::list_notes()?,
        SubCommand::ListTasks {} => bulletlog::list_tasks()?,
        SubCommand::CompleteTask { task_number } => bulletlog::complete_task(task_number)?,
    }

    Ok(())
}

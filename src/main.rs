use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{fs, io};

use chrono::prelude::*;
use clap::Clap;
use mktemp::Temp;
use regex::Regex;

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

fn make_temp_file() -> Result<PathBuf, std::io::Error> {
    let temp_path = Temp::new_file()?;
    let path = temp_path.to_path_buf();
    temp_path.release();
    Ok(path)
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    // FIXME load config file
    let path = Path::new(".BULLETLOG");
    let file = if path.exists() {
        File::open(&path)?
    } else {
        File::create(&path)?
    };

    match opts.subcmd {
        SubCommand::Add { note, date } => {
            let date_str = date.unwrap_or_else(|| today());
            //FIXME Remove unwrap
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?;

            let mut reader = BufReader::new(file);
            let mut buf = String::new();
            reader.read_line(&mut buf)?;
            let mut lines = buf.lines();

            let first_line = lines.next();
            if first_line.is_none() {
                // new file
                let mut file = OpenOptions::new().write(true).open(&path)?;
                write!(file, "## {}\n\n* {}", date_str, note)?;
            } else {
                //FIXME Remove unwrap
                let re = Regex::new(r"^## (\d{4}-\d{2}-\d{2})$").unwrap();
                let cap = re.captures(first_line.unwrap()).unwrap();

                let latest_date = NaiveDate::parse_from_str(&cap[1], "%Y-%m-%d")?;

                if latest_date < date {
                    // New section
                    let temp_path = make_temp_file()?;

                    let mut temp = File::create(&temp_path)?;
                    let mut src = File::open(&path)?;
                    write!(temp, "## {}\n\n* {}\n\n", date_str, note)?;

                    io::copy(&mut src, &mut temp)?;
                    fs::remove_file(&path)?;
                    fs::rename(&temp_path, &path)?;
                } else if latest_date == date {
                    // Add an entry
                } else {
                    // FIXME ATDK
                }
            }
        }
    }

    Ok(())
}

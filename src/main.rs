use std::error::Error;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::{fs, io};

#[macro_use]
extern crate lazy_static;
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
    #[clap(name = "add", aliases = &["a", "note", "n"], about = "Add an note")]
    Note { note: String },
    #[clap(name = "task", alias = "t", about = "Add an task")]
    Task { note: String },

    #[clap(name = "notes", alias = "ns", about = "List all notes")]
    ListNotes {},
    #[clap(name = "tasks", alias = "ts", about = "List all tasks")]
    ListTasks {},
}

const NAIVE_DATE_PATTERN: &str = "%Y-%m-%d";

fn make_temp_file() -> Result<PathBuf, std::io::Error> {
    let temp_path = Temp::new_file()?;
    let path = temp_path.to_path_buf();
    temp_path.release();
    Ok(path)
}

lazy_static! {
    static ref DATE_HEADER_RE: Regex = Regex::new(r"^## (\d{4}-\d{2}-\d{2})$").unwrap();
}

fn get_date_from_header(line: &str) -> NaiveDate {
    let cap = DATE_HEADER_RE
        .captures(&line)
        .expect("Illegal header format");

    NaiveDate::parse_from_str(&cap[1], NAIVE_DATE_PATTERN).expect("Parse Error")
}

fn add_bullet(path: &Path, mark: &str, note: &str) -> Result<(), Box<dyn Error>> {
    let today = Local::today();
    let naive_today = today.naive_local();
    let naive_today_str = Date::format(&today, NAIVE_DATE_PATTERN).to_string();

    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    let mut lines = buf.lines();

    let first_line = lines.next();
    if first_line.is_none() {
        // new file
        let mut file = OpenOptions::new().write(true).open(&path)?;
        write!(file, "## {}\n\n{} {}", naive_today_str, mark, note)?;
    } else {
        let latest_date = get_date_from_header(&first_line.unwrap());

        if latest_date < naive_today {
            // New section
            let temp_path = make_temp_file()?;

            let mut temp = File::create(&temp_path)?;
            let mut src = File::open(&path)?;
            write!(temp, "## {}\n\n{} {}\n\n", naive_today_str, mark, note)?;

            io::copy(&mut src, &mut temp)?;
            fs::remove_file(&path)?;
            fs::rename(&temp_path, &path)?;
        } else if latest_date == naive_today {
            // Add an entry
            let temp_path = make_temp_file()?;
            let temp = File::create(&temp_path)?;
            let mut writer = BufWriter::new(&temp);
            let _ = writer.write_all(buf.as_bytes()); // first line

            let mut appended = false;
            loop {
                let mut buf = String::new();
                let len = reader.read_line(&mut buf)?;
                if len == 0 {
                    break;
                }

                let line = buf.lines().next().unwrap();

                if !appended && DATE_HEADER_RE.is_match(line) {
                    let content = format!("{} {}\n\n", mark, note);
                    let _ = writer.write_all(content.as_bytes());
                    appended = true;
                }

                let _ = writer.write_all(buf.as_bytes());
            }
            writer.flush().unwrap();

            fs::remove_file(&path)?;
            fs::rename(&temp_path, &path)?;
        } else {
            return Err(Box::new(UnsupportedError {}));
        }
    }
    Ok(())
}

fn list_bullets(path: &Path, mark: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);

    let out = io::stdout();
    let mut w = BufWriter::new(out.lock());

    loop {
        let mut buf = String::new();
        let len = reader.read_line(&mut buf)?;
        if len == 0 {
            break;
        }

        if buf.starts_with(mark) {
            write!(w, "{}", buf).unwrap();
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct UnsupportedError;

impl fmt::Display for UnsupportedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsupported")
    }
}

impl std::error::Error for UnsupportedError {}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    // FIXME load config file
    let path = Path::new(".BULLETLOG");
    if path.exists() {
        File::open(&path)?;
    } else {
        File::create(&path)?;
    }

    match opts.subcmd {
        SubCommand::Note { note } => add_bullet(path, "*", &note)?,
        SubCommand::Task { note } => add_bullet(path, "-", &note)?,
        SubCommand::ListNotes {} => list_bullets(path, "*")?,
        SubCommand::ListTasks {} => list_bullets(path, "-")?,
    }

    Ok(())
}

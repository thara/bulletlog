mod errors;
mod utils;

#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use crate::utils::ReadLines;

pub fn add_note(note: &str) -> Result<(), Box<dyn Error>> {
    add_bullet("*", note)?;
    Ok(())
}

pub fn add_task(note: &str) -> Result<(), Box<dyn Error>> {
    add_bullet("-", note)?;
    Ok(())
}

fn add_bullet(mark: &str, note: &str) -> Result<(), Box<dyn Error>> {
    let path = utils::get_logfile_path()?;
    let target_date = utils::get_date()?;

    let target_date_str = target_date.format(utils::NAIVE_DATE_PATTERN).to_string();

    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    let mut lines = buf.lines();

    let first_line = lines.next();
    if first_line.is_none() {
        // new file
        let mut file = OpenOptions::new().write(true).open(&path)?;
        write!(file, "## {}\n\n{} {}\n\n", target_date_str, mark, note)?;
    } else {
        let latest_date = utils::get_date_from_header(&first_line.unwrap());
        if latest_date < target_date {
            // New section
            let mut src = File::open(&path)?;
            utils::write_file(&path, |temp| {
                write!(temp, "## {}\n\n{} {}\n\n", target_date_str, mark, note)?;
                io::copy(&mut src, temp)?;
                Ok(())
            })?;
        } else if latest_date == target_date {
            // Add an entry
            utils::write_file(&path, |temp| {
                let mut writer = BufWriter::new(temp);
                let _ = writer.write_all(buf.as_bytes()); // first line

                let mut appended = false;
                reader.each_lines(|line| {
                    let line = line.lines().next().unwrap();

                    if !appended && utils::DATE_HEADER_RE.is_match(line) {
                        let content = format!("{} {}\n\n", mark, note);
                        let _ = writer.write_all(content.as_bytes());
                        appended = true;
                    }
                    let _ = writer.write_all(buf.as_bytes());
                    Ok(())
                })?;

                if !appended {
                    let content = format!("{} {}\n\n", mark, note);
                    let _ = writer.write_all(content.as_bytes());
                }
                writer.flush().unwrap();

                Ok(())
            })?;
        } else {
            return Err(Box::new(errors::UnsupportedError {}));
        }
    }
    Ok(())
}

pub fn list_notes() -> Result<(), Box<dyn Error>> {
    let mark = "*";
    let path = utils::get_logfile_path()?;
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);

    let out = io::stdout();
    let mut w = BufWriter::new(out.lock());

    reader.each_lines(|line| {
        if line.starts_with(mark) {
            write!(w, "{}", line).unwrap();
        }
        Ok(())
    })?;

    Ok(())
}

pub fn list_tasks() -> Result<(), Box<dyn Error>> {
    let mark = "-";
    let path = utils::get_logfile_path()?;
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);

    let out = io::stdout();
    let mut w = BufWriter::new(out.lock());

    let mut line_number = 0u64;

    reader.each_lines(|line| {
        if line.starts_with(mark) {
            let (_, note) = line.split_at(2);
            write!(w, "{}: {}", line_number, note).unwrap();
            line_number = line_number.wrapping_add(1);
        }
        Ok(())
    })?;

    Ok(())
}

pub fn complete_task(task_number: u64) -> Result<(), Box<dyn Error>> {
    let mark = "-";

    let path = utils::get_logfile_path()?;
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);

    let mut n = 0u64;

    utils::write_file(&path, |temp| {
        let mut writer = BufWriter::new(temp);

        reader.each_lines(|mut line| {
            if line.starts_with(mark) {
                if n == task_number {
                    let (_, note) = line.split_at(2);
                    line = format!("x {}", note);
                }
                n = n.wrapping_add(1);
            }
            let _ = writer.write_all(line.as_bytes());
            Ok(())
        })?;

        writer.flush()?;
        Ok(())
    })?;

    Ok(())
}

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use mktemp::Temp;
use regex::Regex;

pub fn get_logfile_path() -> Result<PathBuf> {
    let pathname = env::var("BULLETLOG_FILE").unwrap_or(".BULLETLOG".to_string());

    let path = Path::new(&pathname);
    if path.exists() {
        File::open(&path)?;
    } else {
        File::create(&path)?;
    }
    Ok(path.to_path_buf())
}

pub const NAIVE_DATE_PATTERN: &str = "%Y-%m-%d";

pub fn get_date() -> Result<NaiveDate> {
    let date = env::var("BULLETLOG_DATE")
        .map(|v| NaiveDate::parse_from_str(&v, NAIVE_DATE_PATTERN).expect("Parse Error"))
        .unwrap_or_else(|_| Local::today().naive_local());
    Ok(date)
}

lazy_static! {
    pub static ref DATE_HEADER_RE: Regex = Regex::new(r"^## (\d{4}-\d{2}-\d{2})$").unwrap();
}

pub fn get_date_from_header(line: &str) -> NaiveDate {
    let cap = DATE_HEADER_RE
        .captures(&line)
        .expect("Illegal header format");

    NaiveDate::parse_from_str(&cap[1], NAIVE_DATE_PATTERN).expect("Parse Error")
}

fn make_tempfile() -> Result<PathBuf> {
    let temp_path = Temp::new_file()?;
    let path = temp_path.to_path_buf();
    temp_path.release();
    Ok(path)
}

pub fn write_file<F>(path: &Path, f: F) -> Result<()>
where
    F: FnOnce(&mut File) -> Result<()>,
{
    let temp_path = make_tempfile()?;
    let mut temp = File::create(&temp_path)?;

    f(&mut temp)?;

    fs::remove_file(&path)?;
    fs::rename(&temp_path, &path)?;
    Ok(())
}

pub trait ReadLines {
    fn each_lines<F>(&mut self, f: F) -> Result<()>
    where
        F: FnMut(String) -> Result<()>;
}

impl ReadLines for BufReader<File> {
    fn each_lines<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(String) -> Result<()>,
    {
        loop {
            let mut buf = String::new();
            let len = self.read_line(&mut buf)?;
            if len == 0 {
                break;
            }
            f(buf)?;
        }
        Ok(())
    }
}

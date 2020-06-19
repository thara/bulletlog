use std::fs::File;
use std::io::BufRead;
use std::io::Result;
use std::io::StdoutLock;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::{fs, io};

use mktemp::Temp;

pub struct Reader {
    reader: BufReader<File>,
}

impl Reader {
    pub fn new(pathname: &str) -> Result<Reader> {
        let path = Path::new(&pathname);
        let file = if path.exists() {
            File::open(&path)?
        } else {
            File::create(&path)?
        };
        let reader = BufReader::new(file);
        Ok(Reader { reader })
    }

    pub fn read_line(&mut self) -> Result<Option<String>> {
        let mut buf = String::new();
        self.reader.read_line(&mut buf)?;

        let first_line = buf.lines().next();
        Ok(first_line.map(|s| s.to_string()))
    }
}

pub trait Writer {
    fn write_line(&mut self, line: &str) -> Result<()>;
    fn finish(&mut self) -> Result<()>;
}

pub struct StdoutWriter<'a> {
    writer: BufWriter<StdoutLock<'a>>,
}

impl<'a> StdoutWriter<'a> {
    pub fn new(out: &'a std::io::Stdout) -> Result<StdoutWriter<'a>> {
        let writer = BufWriter::new(out.lock());
        Ok(StdoutWriter { writer })
    }
}

impl<'a> Writer for StdoutWriter<'a> {
    fn write_line(&mut self, line: &str) -> Result<()> {
        write!(self.writer, "{}", line)?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct FileWriter {
    writer: BufWriter<File>,

    path: String,
    temp_path: PathBuf,
}

impl FileWriter {
    pub fn new(path: &str) -> Result<FileWriter> {
        let temp_path = make_temp_file()?;
        let temp = File::create(&temp_path)?;
        let writer = BufWriter::new(temp);

        let path = path.to_string();
        Ok(FileWriter {
            writer,
            path,
            temp_path,
        })
    }
}

impl Writer for FileWriter {
    fn write_line(&mut self, line: &str) -> Result<()> {
        let _ = self.writer.write_all(line.as_bytes());
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        fs::remove_file(&self.path)?;
        fs::rename(&self.temp_path, &self.path)?;
        Ok(())
    }
}

fn make_temp_file() -> Result<PathBuf> {
    let temp_path = Temp::new_file()?;
    let path = temp_path.to_path_buf();
    temp_path.release();
    Ok(path)
}

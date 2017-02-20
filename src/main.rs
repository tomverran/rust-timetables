extern crate ftp;
extern crate flate2;
extern crate xml;

mod errors;
mod parser;

use errors::{TimetableResult, TimetableError};
use std::error::Error;
use flate2::read::GzDecoder;
use parser::{Train, parse_file};
use ftp::FtpStream;
use ftp::types::FileType;
use std::str;
use std::env;

// given a list of files from FTP, try to find the timetable
fn find_v8_timetable(v: Vec<String>) -> TimetableResult<String> {
    v.into_iter()
        .find(|s| s.ends_with("v8.xml.gz"))
        .ok_or(TimetableError::NoTimetable)
}

fn download_and_parse_file(mut ftp: FtpStream, file: String) -> Result<Vec<Train>, Box<Error>> {
    ftp.transfer_type(FileType::Binary)?;
    ftp.retr(&file, |r| Ok(GzDecoder::new(r).map(parse_file)?))
}

fn run_app() -> Result<(), Box<Error>> {
    let password = env::var("FTP_PASSWORD").map_err(|_| TimetableError::NoPassword)?;
    let mut ftp = FtpStream::connect("datafeeds.nationalrail.co.uk:21")?;
    ftp.login("ftpuser", &password)?;
    
    let timetable = find_v8_timetable(ftp.nlst(None)?)?;
    let trains = download_and_parse_file(ftp, timetable)?;
    Ok(println!("{:?}", trains))
}

fn main() {
    match run_app() {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

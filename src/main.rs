extern crate ftp;
extern crate tempfile;
extern crate flate2;

use ftp::FtpStream;
use ftp::types::FtpError;
use ftp::types::FileType;
use std::io;
use std::str;
use std::env;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Error;
use flate2::read::GzDecoder;
use std::convert::From;

#[derive(Debug)]
enum TimetableError {
    Ftp(FtpError),
    IO(Error),
    NoTimetable
}

impl From<FtpError> for TimetableError {
    fn from(f: FtpError) -> TimetableError {
        TimetableError::Ftp(f)
    }
}

impl From<Error> for TimetableError {
    fn from(f: Error) -> TimetableError {
        TimetableError::IO(f)
    }
}
// the main either type of this entire program
type TimetableResult<A> = Result<A, TimetableError>;


// given a list of files from FTP, try to find the timetable
fn find_v8_timetable(v: Vec<String>) -> TimetableResult<String> {
    v.into_iter()
        .find(|s| s.ends_with("v8.xml.gz"))
        .ok_or(TimetableError::NoTimetable)
}

fn download_file(mut ftp: FtpStream, file: String) -> TimetableResult<File> {
    ftp.transfer_type(FileType::Binary)?;
    return ftp.retr(&file, |r| {
        let mut temp = tempfile::tempfile().map_err(FtpError::ConnectionError)?;
        io::copy(r, Write::by_ref(&mut temp)).map_err(FtpError::ConnectionError)?;
        temp.seek(SeekFrom::Start(0)).map_err(FtpError::ConnectionError)?;
        Result::Ok(temp)
    }).map_err(TimetableError::Ftp);
}

fn main() {

    let password = env::var("FTP_PASSWORD").unwrap();
    let ftp_result = FtpStream::connect("datafeeds.nationalrail.co.uk:21");
    let mut output = File::create("output.xml").unwrap();

    let r = ftp_result
    .map_err(TimetableError::Ftp)
    .and_then(|mut ftp| {
        ftp.login("ftpuser", &password)
        .and_then(|_| ftp.nlst(None))
        .map_err(TimetableError::Ftp)
        .and_then(find_v8_timetable)
        .and_then(|f| download_file(ftp, f))
        .and_then(|f|
            GzDecoder::new(f)
            .map_err(TimetableError::IO)
        )
        .and_then(|mut f|
            io::copy(Read::by_ref(&mut f), Write::by_ref(&mut output))
            .map_err(TimetableError::IO)
        )
    });
    println!("{:?}", r);
}

extern crate ftp;
extern crate flate2;
extern crate xml;

use ftp::FtpStream;
use ftp::types::FtpError;
use ftp::types::FileType;
use std::str;
use std::env;
use std::io::Read;
use std::io::BufReader;
use std::io::Error;
use flate2::read::GzDecoder;
use std::convert::From;
use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
enum TimetableError {
    Ftp(FtpError),
    IO(Error),
    NoTimetable
}

#[derive(Debug)]
struct Train {
    id: String,
    shortlands_time: String,
    victoria_time: String
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

fn download_and_parse_file(mut ftp: FtpStream, file: String) -> TimetableResult<Vec<Train>> {
    ftp.transfer_type(FileType::Binary).map_err(TimetableError::Ftp)?;
    ftp.retr(&file, |r| GzDecoder::new(r).map(parse_file).map_err(FtpError::ConnectionError)) 
    .map_err(TimetableError::Ftp)
}

fn is_shortlands(e: &Vec<OwnedAttribute>) -> bool {
    e.iter()
    .filter(|a| a.name.local_name == "tpl")
    .filter(|a| a.value.starts_with("SHRTLND"))
    .count() != 0
}

fn is_victoria(e: &Vec<OwnedAttribute>) -> bool {
    e.iter()
    .filter(|a| a.name.local_name == "tpl")
    .filter(|a| a.value.starts_with("VIC"))
    .count() != 0
}

fn parse_file<R: Read>(f: R) -> Vec<Train> {
    
    let buf_reader = BufReader::new(f);
    let parser = EventReader::new(buf_reader);
    
    // trains we've found so far
    let mut trains: Vec<Train> = Vec::new();
    
    // the current train we're looking at
    let mut train_id: Option<String> = None;
    let mut shortlands_time: Option<String> = None; 
    let mut terminus_time: Option<String> = None;
    
    for e in parser {
        match e {
            
            Ok(XmlEvent::StartElement { ref name, ref attributes, .. }) if name.local_name == "Journey" => {
                train_id = attributes.into_iter()
                    .find(|a| a.name.local_name == "trainId")
                    .map(|v| v.to_owned().value);
                shortlands_time = None;
                terminus_time = None;
            }
            
            Ok(XmlEvent::StartElement { ref attributes, .. }) if is_victoria(attributes) => {
                terminus_time = attributes.iter().find(|a| a.name.local_name == "pta").map(|v| v.to_owned().value);
            }
            
            Ok(XmlEvent::StartElement { ref attributes, .. }) if is_shortlands(attributes) => {
                shortlands_time = attributes.iter().find(|a| a.name.local_name == "pta").map(|v| v.to_owned().value);
            }
            
            Ok(XmlEvent::EndElement { ref name }) if name.local_name == "Journey" => {
                if shortlands_time.is_some() && terminus_time.is_some() {
                    println!("/{:?} srt: {:?} vic: {:?}", train_id, shortlands_time, terminus_time);
                }
            }
            
            _ => {}
        }
    }
    trains
}

fn main() {
    let password = env::var("FTP_PASSWORD").unwrap();
    let ftp_result = FtpStream::connect("datafeeds.nationalrail.co.uk:21");

    let r = ftp_result
    .map_err(TimetableError::Ftp)
    .and_then(|mut ftp| {
        ftp.login("ftpuser", &password)
        .and_then(|_| ftp.nlst(None))
        .map_err(TimetableError::Ftp)
        .and_then(find_v8_timetable)
        .and_then(|f| download_and_parse_file(ftp, f))
    });
    println!("{:?}", r);
}

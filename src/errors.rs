use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum TimetableError {
    NoTimetable,
    NoPassword
}

impl fmt::Display for TimetableError {
    fn fmt(&self,  f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &TimetableError::NoTimetable => write!(f, "No timetable found"),
            &TimetableError::NoPassword => write!(f, "No FTP_PASSWORD set"),
        }
    }
}

impl Error for TimetableError {
    fn description(&self) -> &str {
        "There was a problem of some kind"
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub type TimetableResult<A> = Result<A, TimetableError>;

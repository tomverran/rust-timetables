use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug)]
pub struct Train {
    id: String,
}


fn is_shortlands(e: &Vec<OwnedAttribute>) -> bool {
    e.iter()
    .filter(|a| a.name.local_name == "tpl")
    .filter(|a| a.value == "BCKNHMJ")
    .count() != 0
}

fn is_victoria(e: &Vec<OwnedAttribute>) -> bool {
    e.iter()
    .filter(|a| a.name.local_name == "tpl")
    .filter(|a| a.value.starts_with("VIC"))
    .count() != 0
}

pub fn parse_file<R: Read>(f: R) -> Vec<Train> {
    
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

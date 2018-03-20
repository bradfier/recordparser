#[macro_use]
extern crate structopt;
extern crate xml;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use structopt::StructOpt;

use xml::reader::{EventReader, XmlEvent};

#[derive(StructOpt, Debug)]
#[structopt(name = "Basic")]
struct Opt {
    /// Output file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
    
    /// Input file
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let file = File::open(opt.input).unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);

    for e in parser {
        match e {
            Ok(e) => match e {
                XmlEvent::StartDocument { .. } =>
                    println!("Started XML Document Parsing"),

                XmlEvent::EndDocument => println!("Finished XML Document Parsing"),
                XmlEvent::StartElement { name, .. } => {
                    println!("Start XML Element Name: {}", name)
                },
                XmlEvent::EndElement { name, .. } => {
                    println!("End XML Element Name: {}", name)
                },
                _ => println!("Unknown element"),
            },
            Err(e) => println!("Error parsing XML document: {}", e)
        }
    }
}

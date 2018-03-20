extern crate chrono;
#[macro_use]
extern crate structopt;

#[macro_use]
extern crate serde_derive;

extern crate csv;
extern crate xml;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::error::Error;

use structopt::StructOpt;
use chrono::prelude::*;

use xml::reader::{EventReader, XmlEvent};
use xml::ParserConfig;

use csv::Writer;

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

#[derive(Debug, Serialize)]
struct Record {
    class: String,
    comment: String,
    laptime: String,
    date: NaiveDateTime,
}

fn parse_records<R: Read>(r: R) -> Vec<Record> {
    enum State {
        Start,
        Records,
        End,
    }

    let mut records: Vec<Record> = vec![];
    let mut state = State::Start;

    let config = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true);

    let parser = EventReader::new_with_config(r, config);

    for e in parser {
        match e {
            Ok(e) => {
                state = match state {
                    State::Start => match e {
                        XmlEvent::StartDocument { .. } => State::Start,
                        XmlEvent::StartElement { ref name, .. }
                            if name.local_name == "TRACKRECORDS" =>
                        {
                            State::Records
                        }
                        _ => panic!("Invalid XML element: {:?}", e),
                    },
                    State::Records => match e {
                        XmlEvent::StartElement {
                            ref name,
                            ref attributes,
                            ..
                        } if name.local_name == "RECORD" =>
                        {
                            let unix_time = &attributes
                                .iter()
                                .find(|&x| x.name.local_name == "date")
                                .expect("Date missing from Record")
                                .value;
                            let record = Record {
                                class: attributes
                                    .iter()
                                    .find(|&x| x.name.local_name == "class")
                                    .expect("Class missing from Record")
                                    .clone()
                                    .value,
                                comment: attributes
                                    .iter()
                                    .find(|&x| x.name.local_name == "comment")
                                    .expect("Comment missing from Record")
                                    .clone()
                                    .value,
                                laptime: attributes
                                    .iter()
                                    .find(|&x| x.name.local_name == "laptime")
                                    .expect("Laptime missing from Record")
                                    .clone()
                                    .value,
                                date: NaiveDateTime::from_timestamp(
                                    (unix_time.parse::<i64>().unwrap()) / 1_000_000,
                                    0,
                                ),
                            };
                            records.push(record);
                            State::Records
                        }
                        XmlEvent::EndElement { .. } => State::Records,
                        XmlEvent::EndDocument { .. } => State::End,
                        _ => panic!("Invalid XML element: {:?}", e),
                    },
                    State::End => break,
                }
            }
            Err(e) => panic!("Error parsing XML due to: {}", e),
        }
    }
    records
}

fn write_output(records: Vec<Record>, path: PathBuf) -> Result<(), Box<Error>> {
    let mut writer = Writer::from_path(path).expect("Couldn't open file for writing.");
    for r in records {
        writer.serialize(r)?;
    }
    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    let file = File::open(opt.input).unwrap();
    let file = BufReader::new(file);

    let records = parse_records(file);
    write_output(records, opt.output).expect("Failure writing out CSV.");
}

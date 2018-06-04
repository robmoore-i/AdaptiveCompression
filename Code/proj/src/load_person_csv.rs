use csv;

use std::error::Error;
use std::io;
use std::process;
use std::env;
use std::ffi::OsString;
use std::fs::File;

// Nodes @ social_network/person_0_0.csv
// Edges @ social_network/person_knows_person_0_0.csv

fn social_network_sf1() -> &'static str {
    return "/home/rob/S3G2/ldbc_snb_datagen/social_network_sf1"
}

fn social_network_sf10() -> &'static str {
    return "/home/rob/S3G2/ldbc_snb_datagen/social_network_sf10"
}

fn nodes(social_network: &str) -> String {
    return social_network.to_string() + "/person_0_0.csv"
}

fn edges(social_network: &str) -> String {
    return social_network.to_string() + "/person_knows_person_0_0.csv"
}

pub fn print_sf10_dirs() {
    println!("nodes @ {}", nodes(social_network_sf1()).to_string());
    println!("edges @ {}", edges(social_network_sf1()).to_string());
}

#[derive(Debug,Deserialize)]
struct RawPerson {
    id: i64,
    firstName: String,
    lastName: String,
    gender: String,
    birthday: String,
    creationDate: String,
    locationIP: String,
    browserUsed: String,
}

struct Person {
    id: i64,
    first_name: String,
    last_name: String,
}

fn read_people(file_path: String) -> Result<Vec<Person>, Box<Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .from_reader(file);
    let mut people: Vec<Person> = Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let raw_person: RawPerson = result?;
        let person = Person { id: raw_person.id, first_name: raw_person.firstName, last_name: raw_person.lastName};
        people.push(person);
    }
    Ok(people)
}

pub fn test() {
    let file_path = nodes(social_network_sf1());
    let read_result = read_people(file_path);
    let people = match read_result {
        Ok(records) => records,
        Err(err)    => panic!(),
    };
    println!("Successfully read {} people", people.len());
}
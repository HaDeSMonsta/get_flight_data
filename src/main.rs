use std::fs::File;
use std::io::Write;

use reqwest::blocking::*;

mod get_json;

fn main() {
    // Read user values
    let name = get_json::get_json_data(get_json::JsonKey::Name);
    let key = get_json::get_json_data(get_json::JsonKey::Key);

    // Format the Simbrief request String
    let simbrief_uri = format!("https://www.simbrief.com/api/xml.fetcher.php?username={name}&json=1");

    // Get Simbrief data via API
    let simbrief_data = send_request(&simbrief_uri);

    // Convert response to JSON datatype
    let simbrief_json: serde_json::Value = serde_json::from_str(simbrief_data.as_str())
        .expect("Simbrief response should be valid JSON");

    let departure_icao = String::from(&simbrief_json["origin"]["icao_code"].to_string());
    let arrival_icao = String::from(&simbrief_json["destination"]["icao_code"].to_string());

    write_file(&String::from("response.json"), &simbrief_data);

    println!("Departure: {}\nArrival: {}", departure_icao, arrival_icao);
}

fn send_request(uri: &String) -> String {
    let http_client = Client::new();
    let response = match http_client.get(uri).send() {
        Ok(data) => {
            match data.text() {
                Ok(val) => val,
                Err(e) => panic!("{e}"),
            }
        }
        Err(e) => panic!("{e}"),
    };
    response
}

fn write_file(name: &String, contend: &String) {

    File::create(name).expect("File should be creatable");
    let file = File::options().append(true).open(name);
    let mut file = match file {
        Ok(val) => val,
        Err(e) => panic!("{e}"),
    };
    write!(&mut file, "{}", contend).expect("File should be writable");
}
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

    let departure_icao = get_icao_from_json(&simbrief_json, true);
    let arrival_icao = get_icao_from_json(&simbrief_json, false);

    println!("Departure: {}\nArrival: {}", departure_icao, arrival_icao);

    // Get METAR
    // Format the departure avwx String
    let avwx_departure_uri = format!("https://avwx.rest/api/metar/{departure_icao}?token={key}");
    let avwx_arrival_uri = format!("https://avwx.rest/api/metar/{arrival_icao}?token={key}");

    // Request the data via API
    let departure_metar = send_request(&avwx_departure_uri);
    let arrival_metar = send_request(&avwx_arrival_uri);

    // Convert to JSON
    let departure_json: serde_json::Value = serde_json::from_str(departure_metar.as_str())
        .expect("Departure response should be valid JSON");
    let arrival_json: serde_json::Value = serde_json::from_str(arrival_metar.as_str())
        .expect("Arrival response should be valid JSON");

    // Get the raw data and flight rules
    // Shadow _metar, because we don't need it anymore
    let departure_metar = get_metar_from_json(&departure_json, true);
    let departure_fr = get_metar_from_json(&departure_json, false);
    let arrival_metar = get_metar_from_json(&arrival_json, true);
    let arrival_fr = get_metar_from_json(&arrival_json, false);

    println!("{departure_metar}\n{departure_fr}\n{arrival_metar}\n{arrival_fr}");
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

fn get_icao_from_json(json: &serde_json::Value, departure: bool) -> String {
    let place = if departure { String::from("origin") }else { String::from("destination") };
    let s = String::from(&json[place]["icao_code"].to_string()[1..5]);
    s
}

fn get_metar_from_json(json: &serde_json::Value, raw: bool)->String{
    let key = if raw { String::from("raw") }else { String::from("flight_rules") };
    let mut s = String::from(&json[key].to_string());
    s = s[1..s.len()-1].to_string();
    s
}
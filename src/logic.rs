use std::fmt::format;
use std::fs::{File, OpenOptions};
use std::io::Write;
use chrono::Local;
use reqwest::blocking::Client;

use crate::json_operations;

pub const LOGFILE_NAME: &str = "gfd.log";

/// Updates and retrieves data regarding departure and arrival airports.
///
/// # Arguments
///
/// * `departure_icao` - The ICAO code of the departure airport.
/// * `arrival_icao` - The ICAO code of the arrival airport.
///
/// # Returns
///
/// A tuple containing the formatted data as strings for the departure and arrival airports.
///
/// # Examples
///
/// ```rust
/// let departure_icao = String::from("EDDB");
/// let arrival_icao = String::from("EHAM");
/// let (print_dep, print_arr) = update_data(&departure_icao, &arrival_icao);
/// assert!(print_dep.contains("Departure ICAO: EDDB"));
/// assert!(print_arr.contains("Arrival ICAO: EHAM"));
/// ```
pub fn update_data(departure_icao: &str, arrival_icao: &str) -> (String, String) {

    // Removed redundant SimBrief call
    // Read user key
    let key = json_operations::get_json_data(json_operations::JsonKey::Key);

    // Get METAR
    // Format the departure avwx String
    let avwx_departure_uri = format!("https://avwx.rest/api/metar/{departure_icao}?token={key}");
    let avwx_arrival_uri = format!("https://avwx.rest/api/metar/{arrival_icao}?token={key}");

    // Request the data via API
    log("Calling avwx API for departure");
    let departure_metar = send_request(&avwx_departure_uri);
    log("Got departure METAR");
    log("Calling avwx API for arrival");
    let arrival_metar = send_request(&avwx_arrival_uri);
    log("Got arrival METAR");

    // Convert to JSON
    let departure_json: serde_json::Value = serde_json::from_str(departure_metar.as_str())
        .expect("Departure response should be valid JSON");
    let arrival_json: serde_json::Value = serde_json::from_str(arrival_metar.as_str())
        .expect("Arrival response should be valid JSON");

    // Get the raw data and flight rules
    // Shadow _metar, because we don't need it anymore
    let (departure_metar, departure_fr) = get_metar_from_json(&departure_json);
    let (arrival_metar, arrival_fr) = get_metar_from_json(&arrival_json);

    // Begin Vatsim block
    // Format URIs
    let vatsim_dep_uri = format!("https://api.t538.net/vatsim/atis/{departure_icao}");
    let vatsim_arr_uri = format!("https://api.t538.net/vatsim/atis/{arrival_icao}");

    // Call the Vatsim API
    log("Calling Vatsim API for departure");
    let dep_atis_response = send_request(&vatsim_dep_uri);
    log("Got departure ATIS");
    log("Calling Vatsim API for arrival ATIS");
    let arr_atis_response = send_request(&vatsim_arr_uri);
    log("Got arrival ATIS");

    // Get the formatted ATIS
    let dep_atis = get_atis(&dep_atis_response, true);
    let arr_atis = get_atis(&arr_atis_response, false);

    // Get the current time so user knows how old information is
    let current_time = Local::now().format("%H:%M");
    let current_time = String::from(format!("Request time: {current_time}"));

    let print_dep = format!("ICAO: {departure_icao}\n\n\
            Vatsim ATIS: {dep_atis}\n\
            METAR: {departure_metar}\n\
            Flight rules: {departure_fr}");

    let print_arr = format!("ICAO: {arrival_icao}\n\n\
            Vatsim ATIS: {arr_atis}\n\
            METAR: {arrival_metar}\n\
            Flight rules: {arrival_fr}");

    let line_separator = String::from("-".repeat(100));

    println!("\n{current_time}\n\n{print_dep}\n\n{line_separator}\n\n{print_arr}");
    (print_dep, print_arr)
}

/// Updates flight plan data from SimBrief API.
/// Retrieves SimBrief username, formats the API request URL,
/// calls the API, converts the response to JSON, and extracts
/// departure and arrival ICAO codes from the JSON response.
///
/// # Returns
///
/// A tuple containing the departure and arrival ICAO codes.
///
/// # Examples
///
/// ```
/// fn main() {
///     let (departure_icao, arrival_icao) = update_fp();
///     println!("Departure ICAO: {}", departure_icao);
///     println!("Arrival ICAO: {}", arrival_icao);
/// }
/// ```
pub fn update_fp() -> (String, String) {

    // Get SimBrief username
    let name = json_operations::get_json_data(json_operations::JsonKey::Name);

    // Format the Simbrief request String
    let simbrief_uri = format!("https://www.simbrief.com/api/xml.fetcher.php?username={name}&json=1");

    // Get Simbrief data via API
    log("Calling Simbrief API");
    let simbrief_data = send_request(&simbrief_uri);
    log("Got response from Simbrief");

    // Convert response to JSON datatype
    let simbrief_json: serde_json::Value = serde_json::from_str(simbrief_data.as_str())
        .expect("Simbrief response should be valid JSON");

    let (departure_icao, arrival_icao) = get_icao_from_json(&simbrief_json);

    (departure_icao, arrival_icao)
}

/// Sends an HTTP GET request to the specified URI and returns the response as a string.
///
/// # Arguments
///
/// * `uri` - The URI to send the GET request to.
///
/// # Panics
///
/// This function will panic if the HTTP request fails or if the response cannot be converted to a string.
///
/// # Examples
///
/// ```
/// let uri = String::from("https://example.com");
/// let response = send_request(&uri);
/// println!("Response: {}", response);
/// ```
fn send_request(uri: &str) -> String {
    // TODO implement error handling
    let http_client = Client::new();
    let response = match http_client.get(uri).send() {
        Ok(data) => {
            match data.text() {
                Ok(val) => val,
                Err(_e) => panic!("{_e}"),
            }
        }
        Err(_e) => panic!("{_e}"),
    };
    response
}

/// Fetches the ICAO codes for the origin and destination airports from a JSON object.
///
/// # Arguments
///
/// * `json` - A reference to a JSON value representing the flight data.
///
/// # Returns
///
/// A tuple containing the ICAO codes for the departure and arrival airports.
///
/// # Examples
///
/// ```
/// use serde_json::json;
///
/// let json_data = json!({
///     "origin": {
///         "icao_code": "EDDB"
///     },
///     "destination": {
///         "icao_code": "EGLL"
///     }
/// });
///
/// let (departure, arrival) = get_icao_from_json(&json_data);
/// assert_eq!(departure, "EDDB");
/// assert_eq!(arrival, "EGLL");
/// ```
fn get_icao_from_json(json: &serde_json::Value) -> (String, String) {
    let mut departure = String::from(&json["origin"]["icao_code"].to_string());
    let mut arrival = String::from(&json["destination"]["icao_code"].to_string());

    departure = trim_icao_str(&departure);
    arrival = trim_icao_str(&arrival);

    (departure, arrival)
}

/// Removes leading and trailing characters from a string
///
/// Given a string, this function trims the string by removing the leading and
/// trailing characters. If the string length is greater than 5, it removes the
/// leading and trailing characters. Otherwise, it returns an empty string.
///
/// # Arguments
///
/// * `s` - A string to trim
///
/// # Returns
///
/// The trimmed string.
fn trim_icao_str(s: &String) -> String {
    if s.len() > 5 {
        s[1..s.len() - 1].to_string()
    } else {
        String::new()
    }
}

/// Extracts the METAR (Meteorological Aerodrome Report) raw and flight rules from a JSON object.
///
/// # Arguments
///
/// * `json` - A reference to a serde_json::Value representing the JSON object.
///
/// # Returns
///
/// A tuple containing the METAR raw and flight rules as strings.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate serde_json;
///
/// use serde_json::Value;
///
/// let json = json!({
///     "raw": "EDDB 251820Z AUTO 24010KT 9999 VCSH SCT027 BKN039 OVC045 FEW///TCU 09/06 Q1005 NOSIG",
///     "flight_rules": "VFR"
/// });
///
/// let (raw, fr) = get_metar_from_json(&json);
///
/// assert_eq!(raw, "EDDB 251820Z AUTO 24010KT 9999 VCSH SCT027 BKN039 OVC045 FEW///TCU 09/06 Q1005 NOSIG");
/// assert_eq!(fr, "VFR");
/// ```
fn get_metar_from_json(json: &serde_json::Value) -> (String, String) {
    let mut raw = json["raw"].to_string();
    let mut fr = json["flight_rules"].to_string();

    raw = raw[1..raw.len() - 1].to_string();
    fr = fr[1..fr.len() - 1].to_string();
    (raw, fr)
}

/// Get the ATIS (Automatic Terminal Information Service) for a specified airport.
///
/// # Arguments
///
/// * `uri` - The URI of the Vatsim API endpoint.
/// * `departure` - A boolean value indicating whether to get the departure ATIS or arrival ATIS.
///
/// # Returns
///
/// A string containing the ATIS information.
///
/// # Panics
///
/// This function panics if the response from the Vatsim API is not a valid JSON array.
///
/// # Examples
///
/// ```rust,no_run
/// let uri = String::from("https://api.t538.net/vatsim/atis/EDDB");
/// let atis = get_atis(&uri, true);
/// println!("{}", atis);
/// ```
fn get_atis(response_raw: &str, departure: bool) -> String {
    if response_raw == "[]" { return "No vatsim ATIS available".to_string(); }

    let dep_or_arr = if departure { String::from("departure") } else { String::from("arrival") };
    let response_arr: serde_json::Value = serde_json::from_str(&response_raw)
        .expect(format!("Response for {dep_or_arr} should be valid JSON Array").as_str());

    let mut to_return = response_arr[0]["text_atis"].to_string();

    if response_arr[1].to_string() != String::from("null") {

        // Get tuples with callsign and ATIS contend
        let zero_tuple = make_atis_tuple(&response_arr, 0);
        let one_tuple = make_atis_tuple(&response_arr, 1);

        // Find the correct ATIS fpr the current situation
        if departure && zero_tuple.0.contains("_D_ATIS") {
            to_return = zero_tuple.1;
        } else if departure && one_tuple.0.contains("_D_ATIS") {
            to_return = one_tuple.1;
        } else if !departure && zero_tuple.0.contains("_A_ATIS") {
            to_return = zero_tuple.1;
        } else if !departure && one_tuple.0.contains("_A_ATIS") {
            to_return = one_tuple.1;
        } else {
            panic!("Neither {}, nor {} contain searched pattern",
                   zero_tuple.0, one_tuple.0);
        }
    }

    to_return = to_return[1..to_return.len() - 1].to_string();

    let atis_arr = to_return.split(",");
    let mut to_return = String::new();

    for slice in atis_arr {
        to_return = to_return + &*String::from(slice[1..slice.len() - 1].to_string()) + &*String::from("\n")
    }

    to_return
}

/// Extracts the callsign and ATIS information from a JSON array.
///
/// # Arguments
///
/// * `json_array` - A reference to a `serde_json::Value` representing the JSON array.
/// * `index` - The index of the element in the JSON array to extract information from.
///
/// # Returns
///
/// A tuple containing the callsign and ATIS information as strings.
///
/// # Example
///
/// ```
/// use serde_json::json;
///
/// let json_array = json!([
///     {
///         "callsign": "EDDF_D_ATIS",
///         "text_atis": "Information ALPHA"
///     },
///     {
///         "callsign": "EDDF_A_ATIS",
///         "text_atis": "Information BRAVO"
///     }
/// ]);
///
/// let (callsign, atis) = make_atis_tuple(&json_array, 1);
/// assert_eq!(callsign, "DEF456");
/// assert_eq!(atis, "Information BRAVO");
/// ```
fn make_atis_tuple(json_array: &serde_json::Value, index: u8) -> (String, String) {
    let callsign = json_array[index as usize]["callsign"].to_string();
    let atis = json_array[index as usize]["text_atis"].to_string();
    (callsign, atis)
}

/// Logs a message along with the current date and time.
///
/// The `log` function prints the message to the console and appends it to a log file.
/// It uses the system's local time to generate the timestamp in the format "[YYYY-MM-DD][HH:MM:SS]".
///
/// # Arguments
///
/// * `message` - A string slice representing the message to be logged.
///
/// # Panics
///
/// The function will panic if it is unable to open the log file for writing.
///
/// # Examples
///
/// ```
/// fn main() {
///     let message = "Error: Something went wrong!";
///     log(message);
/// }
/// ```
pub fn log(message: &str) {
    let now = Local::now().format("[%Y-%m-%d][%H:%M:%S]");
    let mut to_log = format!("{now}: {message}");
    println!("{to_log}");

    let mut log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOGFILE_NAME)
        .expect("Unable to open Logfile");

    to_log.push_str("\n");

    log_file.write(to_log.as_bytes()).expect("Unable to write to Logfile");
}

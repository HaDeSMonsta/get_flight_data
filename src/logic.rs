use std::process::Command;

use chrono::Local;
use reqwest::blocking::Client;

use crate::json_operations;

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
pub fn update_data(departure_icao: &String, arrival_icao: &String) -> (String, String) {

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
    let departure_metar = get_metar_from_json(&departure_json, true);
    let departure_fr = get_metar_from_json(&departure_json, false);
    let arrival_metar = get_metar_from_json(&arrival_json, true);
    let arrival_fr = get_metar_from_json(&arrival_json, false);

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

    let print_dep = format!("Departure ICAO: {departure_icao}\n\n\
            Vatsim ATIS: {dep_atis}\n\
            METAR: {departure_metar}\n\
            Flight rules: {departure_fr}");

    let print_arr = format!("Arrival ICAO: {arrival_icao}\n\n\
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

    let departure_icao = get_icao_from_json(&simbrief_json, true);
    let arrival_icao = get_icao_from_json(&simbrief_json, false);

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
fn send_request(uri: &String) -> String {
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

/// Extracts the International Civil Aviation Organization (ICAO) code from a JSON object.
///
/// This function takes a JSON object (`json`) and a boolean value (`departure`) as arguments.
/// If `departure` is true, it extracts the ICAO code from the "origin" field of the JSON object.
/// Otherwise, it extracts the ICAO code from the "destination" field of the JSON object.
///
/// # Arguments
///
/// * `json` - A reference to the serde_json::Value containing the JSON object.
/// * `departure` - A boolean value indicating whether to extract from "origin" (true) or "destination" (false).
///
/// # Returns
///
/// The extracted ICAO code as a String.
///
/// # Example
///
/// ```
/// use serde_json::json;
/// let json = json!({
///     "origin": {
///         "icao_code": "EDDB"
///     },
///     "destination": {
///         "icao_code": "EHAM"
///     }
/// });
///
/// let departure_icao = get_icao_from_json(&json, true);
/// assert_eq!(departure_icao, "EDDB");
///
/// let destination_icao = get_icao_from_json(&json, false);
/// assert_eq!(destination_icao, "EHAM");
/// ```
fn get_icao_from_json(json: &serde_json::Value, departure: bool) -> String {
    let place = if departure { String::from("origin") } else { String::from("destination") };
    let s = String::from(&json[place]["icao_code"].to_string()[1..5]);
    s
}

/// Extracts the METAR value from the given JSON object.
///
/// # Arguments
///
/// * `json` - A reference to a `serde_json::Value` containing the JSON object.
/// * `raw` - A boolean value indicating whether the METAR value should be returned in raw format.
///
/// # Returns
///
/// A `String` containing the extracted METAR value.
///
/// # Example
///
/// ```rust
/// let json = json!({"flight_rules": "IFR", "raw": "METAR data"});
/// let result = get_metar_from_json(&json, false);
/// assert_eq!(result, "IFR");
/// ```
fn get_metar_from_json(json: &serde_json::Value, raw: bool) -> String {
    let key = if raw { String::from("raw") } else { String::from("flight_rules") };
    let mut s = json[key].to_string();
    s = s[1..s.len() - 1].to_string();
    s
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
fn get_atis(response_raw: &String, departure: bool) -> String {
    if response_raw == "[]" { return "No vatsim ATIS available".to_string(); }

    let dep_or_arr = if departure { String::from("departure") } else { String::from("arrival") };
    let response_arr: serde_json::Value = serde_json::from_str(&response_raw.as_str())
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

/// Logs a message with timestamp in the format '[YYYY-MM-DD][HH:MM:SS]: {message}'
///
/// # Arguments
///
/// * `message` - The message to be logged
///
/// # Example
///
/// ```
/// log("This is a log message");
/// ```
pub fn log(message: &str) {
    let now = Local::now().format("[%Y-%m-%d][%H:%M:%S]");
    println!("{now}: {message}")
}

/// Clears the terminal screen.
///
/// This function clears the terminal screen based on the current operating system.
/// If the operating system is Windows, it uses the "cls" command to clear the screen.
/// If the operating system is Unix, it uses the "clear" command to clear the screen.
/// If the operating system cannot be detected, it panics with an error message.
///
/// # Example
///
/// ```rust
/// clear_term();
/// ```
fn clear_term() {
    if cfg!(windows) {
        Command::new("cmd")
            .args(&mut ["/C", "cls"])
            .spawn()
            .expect("Command should be executable");
    } else if cfg!(unix) {
        Command::new("clear").spawn().expect("Command should be executable");
    } else { panic!("Couldn't detect OS") }
}

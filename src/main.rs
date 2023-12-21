use reqwest::blocking::*;

mod get_json;

fn main() {
    // Read user values
    let name = get_json::get_json_data(get_json::JsonKey::Name);
    let key = get_json::get_json_data(get_json::JsonKey::Key);

    // Format the Simbrief request String
    let simbrief_uri = format!("https://www.simbrief.com/api/xml.fetcher.php?username={name}&json=1");

    // Get Simbrief data via API
    println!("Calling Simbrief API");
    let simbrief_data = send_request(&simbrief_uri);
    println!("Got response from Simbrief");

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
    println!("Calling avwx API for departure");
    let departure_metar = send_request(&avwx_departure_uri);
    println!("Got departure METAR\nCalling avwx API for arrival");
    let arrival_metar = send_request(&avwx_arrival_uri);
    println!("Gor arrival METAR");

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

    // Get the formatted ATIS
    let dep_atis = get_atis(&vatsim_dep_uri, true);
    let arr_atis = get_atis(&vatsim_arr_uri, false);

    let print_dep = format!("Departure ICAO: {departure_icao}\n\n\
    Vatsim ATIS: {dep_atis}\n\
    METAR: {departure_metar}\n\
    Flight rules: {departure_fr}");

    let print_arr = format!("Arrival ICAO: {arrival_icao}\n\n\
    Vatsim ATIS: {arr_atis}\n\
    METAR: {arrival_metar}\n\
    Flight rules: {arrival_fr}");

    let line_separator = String::from("-".repeat(100));

    println!("\n{print_dep}\n\n{line_separator}\n\n{print_arr}")
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
/// let uri = String::from("https://vatsim.net/api/atis/EDDF/departure");
/// let atis = get_atis(&uri, true);
/// println!("{}", atis);
/// ```
fn get_atis(uri: &String, departure: bool) -> String {
    let dep_or_arr = if departure { String::from("departure") } else { String::from("arrival") };
    println!("Calling Vatsim API for {dep_or_arr}");
    let response_raw = send_request(uri);
    println!("Got ATIS for {dep_or_arr}");
    if response_raw == "[]" { return String::from("No vatsim ATIS available"); }

    let response_arr: serde_json::Value = serde_json::from_str(&response_raw.as_str())
        .expect(format!("Response from {uri} should be valid JSON Array").as_str());

    let mut to_return = response_arr[0]["text_atis"].to_string();

    if response_arr[1].to_string() != String::from("null") {
        if departure {
            to_return = response_arr[0]["text_atis"].to_string();
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
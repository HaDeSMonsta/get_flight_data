use std::fs::File;

pub enum JsonKey {
    Name,
    Key,
}

/// Retrieves the value of a JSON key from a file.
///
/// # Arguments
///
/// * `key` - The JSON key to retrieve the value for.
///
/// # Panics
///
/// Panics if the file "userdata.json" cannot be opened or if the file
/// does not contain valid JSON.
///
/// # Returns
///
/// Returns the value associated with the given key as a String. The value
/// is obtained by reading the contents of the file "userdata.json" and
/// extracting the value corresponding to the provided key.
///
/// # Examples
///
/// ```
/// use serde_json::Value;
///
/// enum JsonKey {
///     Name,
///     Key,
/// }
///
/// let json_data = get_json_data(JsonKey::Name);
///
/// assert_eq!(json_data, "John Doe");
/// ```
pub fn get_json_data(key: JsonKey) -> String {
    let file = match File::open("userdata.json") {
        Ok(f) => f,
        Err(_e) => panic!("{_e}"),
    };

    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("File should be proper JSON");

    let val = match key {
        JsonKey::Name => json.get("simBrief_userName").unwrap().to_string(),
        JsonKey::Key => json.get("api_token").unwrap().to_string(),
    };

    val[1..val.len() - 1].to_string()
}

use std::fs::{File, OpenOptions};
use std::io::Write;
use serde_json::{from_reader, Value};

const NAME_FIELD: &str = "simBrief_userName";
const KEY_FIELD: &str = "api_token";
const FILE_PATH: &str = "userdata.json";

pub enum JsonKey {
    Name,
    Key,
}

/// Retrieves JSON data from a given key in a file. If the file does not exist or is not a valid JSON format,
/// it creates a new JSON file with empty values but valid keys.
///
/// # Arguments
///
/// * `key` - An enum value representing the key to retrieve from the JSON data.
///
/// # Errors
///
/// This function returns an empty string under the following conditions:
/// * If the file does not exist, it is created anew with empty values but valid keys.
/// * If the file exists but cannot be read or written.
/// * If the file is not a valid JSON file, it is overwritten with a valid JSON structure containing empty values.
///
/// # Examples
///
/// ```rust
/// let json_data = get_json_data(JsonKey::Name);
/// assert_eq!(json_data, "value");
/// ```
pub fn get_json_data(key: JsonKey) -> String {
    let file = OpenOptions::new()
        .read(true) // Read access
        .write(true) // Write access
        .create(true) // Create if it does not exist
        .open(&FILE_PATH) // And finally open
        .unwrap_or_else(|e| panic!("Error opening {}\nError: {e}", FILE_PATH));

    let result: Result<Value, _> = from_reader(&file);
    let json;

    match result {
        Ok(value) => json = value,
        Err(_) => {
            println!("Error reading from file {FILE_PATH}, will create it");
            drop(file);
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&FILE_PATH)
                .expect("Unable to open invalid file again");
            let default_json = String::from(format!(
                "{{\n\
                \t\"{NAME_FIELD}\": \"\",\n\
                \t\"{KEY_FIELD}\": \"\"\n\
                }}"
            ));
            write!(file, "{default_json}").expect("Failed writing to file after seeing it's not proper json");
            return String::new();
        }
    }

    let val = match key {
        JsonKey::Name => json.get(NAME_FIELD).unwrap().to_string(),
        JsonKey::Key => json.get(KEY_FIELD).unwrap().to_string(),
    };

    val[1..val.len() - 1].to_string()
}

/// Sets the JSON data based on a given key and value.
///
/// # Arguments
///
/// * `key` - The key indicating which field to update.
/// * `val` - The new value to set for the specified field.
///
/// # Panics
///
/// This function will panic if it is unable to create or write to the file specified by `FILE_PATH`.
///
/// # Examples
///
/// ```
/// use my_module::JsonKey;
///
/// let key = JsonKey::Name;
/// let val = String::from("John Doe");
///
/// my_module::set_json_data(key, val);
/// ```
pub fn set_json_data(key: JsonKey, val: &str) {
    let other_val = match key {
        JsonKey::Name => get_json_data(JsonKey::Key),
        JsonKey::Key => get_json_data(JsonKey::Name),
    };

    let contend = match key {
        JsonKey::Name => {
            String::from(format!(
                "{{\n\
                \t\"{NAME_FIELD}\": \"{val}\",\n\
                \t\"{KEY_FIELD}\": \"{other_val}\"\n\
                }}"
            ))
        }
        JsonKey::Key => {
            String::from(format!(
                "{{\n\
                \t\"{NAME_FIELD}\": \"{other_val}\",\n\
                \t\"{KEY_FIELD}\": \"{val}\"\n\
                }}"
            ))
        }
    };

    let mut file = File::create(&FILE_PATH).expect("Unable to create file");
    write!(file, "{contend}").expect("Unable to write to file");
}
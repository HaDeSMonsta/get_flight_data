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
    let mut file = OpenOptions::new()
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
pub fn set_json_data(key: JsonKey, val: String) {
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
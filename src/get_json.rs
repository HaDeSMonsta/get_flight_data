use std::fs::File;
use std::io::Write;

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
    let file = match File::open(&FILE_PATH) {
        Ok(f) => f,
        Err(_e) => panic!("{_e}"),
    };

    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("File should be proper JSON");

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
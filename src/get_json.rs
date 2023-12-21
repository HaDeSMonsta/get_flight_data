use std::fs::File;

pub enum JsonKey {
    Name,
    Key,
}

pub fn get_json_data(key: JsonKey) -> String {
    let file = match File::open("userdata.json") {
        Ok(f) => f,
        Err(e) => panic!("{e}"),
    };

    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("File should be proper JSON");

    let val = match key {
        JsonKey::Name => json.get("simBrief_userName").unwrap().to_string(),
        JsonKey::Key => json.get("api_token").unwrap().to_string(),
    };

    val[1..val.len() - 1].to_string()
}

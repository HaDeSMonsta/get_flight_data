mod get_json;

fn main() {
    let name = get_json::get_json_data(get_json::JsonKey::Name);
    let key = get_json::get_json_data(get_json::JsonKey::Key);

    println!("Name: {name}\nKey: {key}");
    println!("Length: {}", name.len());
}

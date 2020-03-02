pub fn camelcase_to_dashed(property_name: &str) -> String {
    let mut res = String::with_capacity(property_name.len() * 2);
    for character in property_name.chars() {
        if character.is_uppercase() {
            res.push('-');
            res.push_str(&character.to_lowercase().to_string());
        } else {
            res.push(character);
        }
    }
    res
}
